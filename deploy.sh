#!/usr/bin/env bash
# ==============================================================================
# Cloudflare Serverless Deployment Build Pipeline Script (Pages & Workers)
# Universal Rust Algebra Engine (URAE)
#
# Usage:
#   ./deploy.sh                  # Pure Static Site (Default, no Edge Worker API)
#   ./deploy.sh --static-only    # Explicit Pure Static Site
#   ./deploy.sh --with-api       # Full-Stack Mode (Static Site + Cloudflare Pages Worker API)
#   ./deploy.sh --worker-deploy  # Standalone Cloudflare Worker Deploy (Wrangler)
# ==============================================================================
set -euo pipefail

ENABLE_EDGE_API="${ENABLE_EDGE_API:-false}"
CLOUDFLARE_WORKER_DEPLOY="${CLOUDFLARE_WORKER_DEPLOY:-false}"

for arg in "$@"; do
    case "$arg" in
        --static-only)
            ENABLE_EDGE_API="false"
            CLOUDFLARE_WORKER_DEPLOY="false"
            ;;
        --with-api)
            ENABLE_EDGE_API="true"
            ;;
        --worker-deploy)
            CLOUDFLARE_WORKER_DEPLOY="true"
            ;;
    esac
done

if [ "$ENABLE_EDGE_API" = "true" ]; then
    echo "=== Initializing Cloudflare Build: Full-Stack Mode (Static Site + Edge Worker API) ==="
elif [ "$CLOUDFLARE_WORKER_DEPLOY" = "true" ]; then
    echo "=== Initializing Cloudflare Build: Standalone Wrangler Worker Mode ==="
else
    echo "=== Initializing Cloudflare Build: Pure Static Site Mode (No Serverless API) ==="
fi

# 1. Persistent Environment & PATH Setup
export NODE_ENV="production"

if [ -d "/opt/buildhome" ]; then
    export CARGO_HOME="/opt/buildhome/.cargo"
    export RUSTUP_HOME="/opt/buildhome/.rustup"
else
    export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
    export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
fi

export PATH="$CARGO_HOME/bin:$HOME/.cargo/bin:/opt/buildhome/.cargo/bin:$PATH"
mkdir -p "$CARGO_HOME/bin"

if [ -f "$CARGO_HOME/env" ]; then
    . "$CARGO_HOME/env"
elif [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi

# 2. Rust Toolchain & Target Verification
RUST_TOOLCHAIN="1.95.0"

if ! command -v rustup &> /dev/null && [ ! -f "$CARGO_HOME/bin/rustup" ]; then
    echo "Rust compiler not detected. Installing Rust $RUST_TOOLCHAIN minimal toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain "$RUST_TOOLCHAIN" --profile minimal --target wasm32-unknown-unknown
    if [ -f "$CARGO_HOME/env" ]; then
        . "$CARGO_HOME/env"
    fi
else
    echo "Rust toolchain detected: $(rustc --version || echo 'Active')"
    if command -v rustup &> /dev/null; then
        rustup toolchain install "$RUST_TOOLCHAIN" --profile minimal --target wasm32-unknown-unknown 2>/dev/null || true
        rustup default "$RUST_TOOLCHAIN" 2>/dev/null || true
        rustup target add wasm32-unknown-unknown 2>/dev/null || true
    elif [ -f "$CARGO_HOME/bin/rustup" ]; then
        "$CARGO_HOME/bin/rustup" toolchain install "$RUST_TOOLCHAIN" --profile minimal --target wasm32-unknown-unknown 2>/dev/null || true
        "$CARGO_HOME/bin/rustup" default "$RUST_TOOLCHAIN" 2>/dev/null || true
        "$CARGO_HOME/bin/rustup" target add wasm32-unknown-unknown 2>/dev/null || true
    fi
fi

# 3. wasm-bindgen CLI Installation (Matches Cargo.lock exact version for zero linker errors)
WASM_BINDGEN_VERSION=$(grep -A 1 'name = "wasm-bindgen"' Cargo.lock 2>/dev/null | grep 'version =' | head -n 1 | cut -d '"' -f 2 || echo "0.2.115")
if [ -z "$WASM_BINDGEN_VERSION" ]; then
    WASM_BINDGEN_VERSION="0.2.115"
fi

WASM_BINDGEN_BIN="$CARGO_HOME/bin/wasm-bindgen"
if [ -x "$WASM_BINDGEN_BIN" ] && "$WASM_BINDGEN_BIN" --version 2>&1 | grep -q "$WASM_BINDGEN_VERSION"; then
    echo "Cached wasm-bindgen detected: $("$WASM_BINDGEN_BIN" --version)"
elif command -v wasm-bindgen &> /dev/null && wasm-bindgen --version 2>&1 | grep -q "$WASM_BINDGEN_VERSION"; then
    echo "System wasm-bindgen detected: $(wasm-bindgen --version)"
    WASM_BINDGEN_BIN="wasm-bindgen"
else
    echo "Downloading and caching wasm-bindgen v${WASM_BINDGEN_VERSION} CLI binary..."
    temp_wb="/tmp/wasm-bindgen-${WASM_BINDGEN_VERSION}.tar.gz"
    wget -qO "$temp_wb" "https://github.com/rustwasm/wasm-bindgen/releases/download/${WASM_BINDGEN_VERSION}/wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl.tar.gz" || \
    wget -qO "$temp_wb" "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${WASM_BINDGEN_VERSION}/wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl.tar.gz" || true
    if [ -f "$temp_wb" ] && [ -s "$temp_wb" ]; then
        tar -xzf "$temp_wb" -C /tmp
        find /tmp -name "wasm-bindgen" -type f -exec mv {} "$CARGO_HOME/bin/wasm-bindgen" \;
        chmod +x "$CARGO_HOME/bin/wasm-bindgen"
        rm -rf "$temp_wb" /tmp/wasm-bindgen*
    fi
    if [ ! -x "$CARGO_HOME/bin/wasm-bindgen" ]; then
        echo "Precompiled binary unavailable. Installing wasm-bindgen-cli via cargo..."
        cargo install wasm-bindgen-cli --version "$WASM_BINDGEN_VERSION" --root "$CARGO_HOME"
    fi
    WASM_BINDGEN_BIN="$CARGO_HOME/bin/wasm-bindgen"
    echo "wasm-bindgen installed: $("$WASM_BINDGEN_BIN" --version || echo 'Ready')"
fi

# 4. Binaryen (wasm-opt) (Check Cache & Validate Execution)
WASM_OPT_BIN="$CARGO_HOME/bin/wasm-opt"

if [ -x "$WASM_OPT_BIN" ] && "$WASM_OPT_BIN" --version &> /dev/null; then
    echo "Cached wasm-opt detected: $("$WASM_OPT_BIN" --version)"
elif command -v wasm-opt &> /dev/null && wasm-opt --version &> /dev/null; then
    echo "System wasm-opt detected: $(wasm-opt --version)"
    WASM_OPT_BIN="wasm-opt"
else
    echo "Downloading and caching Binaryen wasm-opt..."
    BINARYEN_VERSION="version_122"
    temp_tar="/tmp/binaryen-${BINARYEN_VERSION}.tar.gz"
    wget -qO "$temp_tar" "https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VERSION}/binaryen-${BINARYEN_VERSION}-x86_64-linux.tar.gz" || \
    wget -qO "$temp_tar" "https://github.com/WebAssembly/binaryen/releases/latest/download/binaryen-x86_64-linux.tar.gz"
    tar -xzf "$temp_tar" -C /tmp
    find /tmp -name "wasm-opt" -type f -exec mv {} "$CARGO_HOME/bin/wasm-opt" \;
    chmod +x "$CARGO_HOME/bin/wasm-opt"
    rm -rf "$temp_tar" /tmp/binaryen*
    WASM_OPT_BIN="$CARGO_HOME/bin/wasm-opt"
    echo "wasm-opt installed: $("$WASM_OPT_BIN" --version || echo 'Ready')"
fi

# 5. Clean & Build WebAssembly Application
echo "Purging previous build distribution caches..."
rm -rf crates/urae-wasm/public/pkg crates/urae-wasm/pkg dist

export RUSTFLAGS="-C target-feature=+bulk-memory,+mutable-globals,+nontrapping-fptoint,+sign-ext ${RUSTFLAGS:-}"

echo "Compiling WebAssembly release with Cargo (toolchain: $RUST_TOOLCHAIN, bulk-memory enabled)..."
if command -v rustup &> /dev/null; then
    rustup run "$RUST_TOOLCHAIN" cargo build --target wasm32-unknown-unknown -p urae-wasm --release
elif [ -f "$CARGO_HOME/bin/rustup" ]; then
    "$CARGO_HOME/bin/rustup" run "$RUST_TOOLCHAIN" cargo build --target wasm32-unknown-unknown -p urae-wasm --release
else
    cargo build --target wasm32-unknown-unknown -p urae-wasm --release
fi

echo "Generating WebAssembly bindings via wasm-bindgen..."
mkdir -p crates/urae-wasm/public/pkg
if [ -x "$WASM_BINDGEN_BIN" ] || command -v wasm-bindgen &> /dev/null; then
    "${WASM_BINDGEN_BIN:-wasm-bindgen}" target/wasm32-unknown-unknown/release/urae_wasm.wasm --target web --out-dir crates/urae-wasm/public/pkg
else
    cd crates/urae-wasm
    wasm-pack build --target web --out-dir public/pkg --release
    cd ../..
fi

DIST_DIR="crates/urae-wasm/public"

# Run wasm-opt pass on generated wasm artifact with full performance optimizations
WASM_OPT_FLAGS=(
    "-Oz"
    "--enable-bulk-memory"
    "--enable-bulk-memory-opt"
    "--enable-mutable-globals"
    "--enable-sign-ext"
    "--enable-nontrapping-float-to-int"
    "--enable-reference-types"
    "--enable-multivalue"
    "--strip-debug"
    "--strip-producers"
)

if [ -x "$WASM_OPT_BIN" ] || command -v wasm-opt &> /dev/null; then
    for wasm_file in "$DIST_DIR"/pkg/*.wasm; do
        if [ -f "$wasm_file" ]; then
            echo "Optimizing WASM with wasm-opt (bulk-memory, fast math & performance flags): $wasm_file"
            "$WASM_OPT_BIN" "${WASM_OPT_FLAGS[@]}" "$wasm_file" -o "$wasm_file" || "$WASM_OPT_BIN" -Oz "$wasm_file" -o "$wasm_file" || true
        fi
    done
fi

# 6. Production Asset Minification (HTML, CSS, JS)
if [ -d "$DIST_DIR" ]; then
    echo "=== Running Production Asset Minification (HTML, CSS, JS) for '$DIST_DIR' ==="

    if command -v npx &> /dev/null; then
        echo "Minifying JavaScript and CSS assets using esbuild..."
        for js_file in "$DIST_DIR"/*.js "$DIST_DIR"/pkg/*.js; do
            if [ -f "$js_file" ]; then
                echo "  Minifying JS: $js_file"
                npx --yes esbuild "$js_file" --minify --allow-overwrite --outfile="$js_file" 2>/dev/null || true
            fi
        done
        for css_file in "$DIST_DIR"/*.css; do
            if [ -f "$css_file" ]; then
                echo "  Minifying CSS: $css_file"
                npx --yes esbuild "$css_file" --minify --allow-overwrite --outfile="$css_file" 2>/dev/null || true
            fi
        done
        if [ -f "$DIST_DIR/index.html" ]; then
            echo "  Minifying HTML: $DIST_DIR/index.html"
            npx --yes html-minifier-terser --collapse-whitespace --remove-comments --remove-redundant-attributes --remove-script-type-attributes --remove-style-link-type-attributes --use-short-doctype --minify-css true --minify-js true -o "$DIST_DIR/index.html" "$DIST_DIR/index.html" 2>/dev/null || true
        fi
    elif command -v python3 &> /dev/null; then
        echo "Node/npx not available. Using Python minification engine fallback..."
        python3 -c '
import os, re, sys, glob

dist_dir = sys.argv[1]

def minify_css(content):
    content = re.sub(r"/\*[\s\S]*?\*/", "", content)
    content = re.sub(r"\s+", " ", content)
    content = re.sub(r"\s*([\{\}:;,])\s*", r"\1", content)
    content = content.replace(";}", "}")
    return content.strip()

def minify_js(content):
    lines = []
    for line in content.splitlines():
        stripped = line.strip()
        if stripped.startswith("//") and not stripped.startswith("///"):
            continue
        lines.append(line)
    content = "\n".join(lines)
    content = re.sub(r"/\*[\s\S]*?\*/", "", content)
    content = re.sub(r"[ \t]+", " ", content)
    content = re.sub(r"\n\s*", "\n", content)
    content = re.sub(r"\s*([=+\-*/%&|!<>?:,;{}()[\]])\s*", r"\1", content)
    return content.strip()

for fpath in glob.glob(os.path.join(dist_dir, "*.css")):
    try:
        with open(fpath, "r", encoding="utf-8") as f:
            c = f.read()
        with open(fpath, "w", encoding="utf-8") as f:
            f.write(minify_css(c))
        print(f"  Minified CSS: {fpath}")
    except Exception as e:
        print(f"  Error minifying {fpath}: {e}")

for fpath in glob.glob(os.path.join(dist_dir, "**", "*.js"), recursive=True):
    try:
        with open(fpath, "r", encoding="utf-8") as f:
            c = f.read()
        with open(fpath, "w", encoding="utf-8") as f:
            f.write(minify_js(c))
        print(f"  Minified JS: {fpath}")
    except Exception as e:
        print(f"  Error minifying {fpath}: {e}")

html_path = os.path.join(dist_dir, "index.html")
if os.path.exists(html_path):
    try:
        with open(html_path, "r", encoding="utf-8") as f:
            html = f.read()
        html = re.sub(r"<!--(?!\[if)[\s\S]*?-->", "", html)
        html = re.sub(r"<style[^>]*>([\s\S]*?)</style>", lambda m: f"<style>{minify_css(m.group(1))}</style>", html, flags=re.IGNORECASE)
        html = re.sub(r">\s+<", "><", html)
        html = re.sub(r"[ \t]+", " ", html)
        with open(html_path, "w", encoding="utf-8") as f:
            f.write(html.strip())
        print(f"  Minified HTML: {html_path}")
    except Exception as e:
        print(f"  Error processing {html_path}: {e}")
' "$DIST_DIR"
    fi

    # 7. High-Ratio Asset Pre-Compression (Brotli Level 11 + Gzip Level 9)
    echo "=== Generating Pre-Compressed Brotli (.br) & Gzip (.gz) Assets ==="
    if command -v python3 &> /dev/null; then
        python3 -c '
import os, sys, gzip, glob

dist_dir = sys.argv[1]
target_files = []
for ext in ("*.wasm", "*.js", "*.css", "*.html", "*.json", "*.svg"):
    target_files.extend(glob.glob(os.path.join(dist_dir, ext)))
    target_files.extend(glob.glob(os.path.join(dist_dir, "**", ext), recursive=True))

# 1. Gzip Level 9
for fpath in set(target_files):
    gz_path = fpath + ".gz"
    try:
        with open(fpath, "rb") as f_in, gzip.open(gz_path, "wb", compresslevel=9) as f_out:
            f_out.write(f_in.read())
    except Exception as e:
        print(f"  Gzip failed for {fpath}: {e}")

# 2. Brotli Level 11 (if brotli module is available)
try:
    import brotli
    for fpath in set(target_files):
        br_path = fpath + ".br"
        with open(fpath, "rb") as f_in:
            data = f_in.read()
        compressed = brotli.compress(data, quality=11, mode=brotli.MODE_GENERIC)
        with open(br_path, "wb") as f_out:
            f_out.write(compressed)
    print("  Successfully pre-compressed assets with Brotli (q11) & Gzip (level 9)")
except ImportError:
    print("  Pre-compressed assets with Gzip (level 9). Brotli CLI check...")
' "$DIST_DIR" || true
        if command -v brotli &> /dev/null; then
            for fpath in "$DIST_DIR"/*.{wasm,js,css,html,json,svg} "$DIST_DIR"/pkg/*.{wasm,js}; do
                if [ -f "$fpath" ] && [ ! -f "${fpath}.br" ]; then
                    brotli -f -k -q 11 "$fpath" 2>/dev/null || true
                fi
            done
        fi
    fi

    # 8. Configure Cloudflare Deployment Mode (Static-Only vs Full-Stack Edge Worker)
    cp -f crates/urae-wasm/public/_headers "$DIST_DIR/_headers" 2>/dev/null || true
    cp -f crates/urae-wasm/public/_redirects "$DIST_DIR/_redirects" 2>/dev/null || true

    if [ "$ENABLE_EDGE_API" = "true" ]; then
        echo "Configuring Cloudflare Pages Edge Worker (_worker.js & _routes.json)..."
        # Adapt relative import paths for _worker.js when placed inside public/
        sed -e 's|\./public/pkg/|\./pkg/|g' crates/urae-wasm/worker.js > "$DIST_DIR/_worker.js"

        cat << 'EOF' > "$DIST_DIR/_routes.json"
{
  "version": 1,
  "include": [
    "/api/*"
  ],
  "exclude": [
    "/pkg/*",
    "/index.html",
    "/favicon.ico",
    "/manifest.json",
    "/sw.js",
    "/env.js",
    "/*.wasm",
    "/*.js",
    "/*.css"
  ]
}
EOF
        echo "Edge Worker API enabled for /api/* routes."
    else
        # Ensure pure static site deployment: remove any worker artifacts
        rm -f "$DIST_DIR/_worker.js" "$DIST_DIR/_routes.json"
        echo "Pure static site deployment confirmed (Zero serverless worker overhead)."
    fi
fi

echo "=== Build Completed Successfully! Output directory ready in: '$DIST_DIR' ==="

# 9. Deployment Context Router
if [ "$CLOUDFLARE_WORKER_DEPLOY" = "true" ]; then
    echo "Wrangler Worker deployment context detected."
    if ! command -v wrangler &> /dev/null; then
        if command -v npm &> /dev/null; then
            echo "Installing Cloudflare Wrangler globally via npm..."
            npm install -g wrangler
        else
            echo "ERROR: npm is required to install Wrangler for Worker deployments."
            exit 1
        fi
    fi
    echo "Executing Wrangler Deploy..."
    cd crates/urae-wasm
    wrangler deploy
    cd ../..
else
    echo "Pages / Static CDN deployment context detected. Ready for publishing to Cloudflare Pages."
fi
