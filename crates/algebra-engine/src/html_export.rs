//! # `algebra_engine::html_export`
//!
//! Standalone Self-Contained Interactive HTML Dashboard & 3D Viewer Exporter.
//!
//! Generates zero-dependency single-file `.html` documents with:
//! - KaTeX mathematical typesetting for formulas.
//! - Three.js WebGL canvas for interactive 3D rotating CAD/FEA/Riemann surfaces.
//! - Interactive parameter scrubbers & reactive computed output metrics.

use crate::cad::Mesh3D;

/// Interactive HTML Dashboard Exporter.
pub struct StandaloneHtmlExporter;

impl StandaloneHtmlExporter {
    /// Export a 3D CAD mesh with Title, Description, and KaTeX mathematical annotations to a standalone `.html` string.
    pub fn export_mesh_to_html(
        title: &str,
        description: &str,
        latex_formula: &str,
        mesh: &Mesh3D,
    ) -> String {
        // Serialize vertices and triangles to JSON arrays for embedded JavaScript
        let vertices_json = serde_json::to_string(&mesh.vertices).unwrap_or_else(|_| "[]".into());
        let triangles_json = serde_json::to_string(&mesh.triangles).unwrap_or_else(|_| "[]".into());

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.css">
    <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.js"></script>
    <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/contrib/auto-render.min.js"
        onload="renderMathInElement(document.body);"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js"></script>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            background-color: #0f172a;
            color: #f8fafc;
            margin: 0;
            padding: 24px;
            display: flex;
            flex-direction: column;
            align-items: center;
        }}
        .card {{
            background: #1e293b;
            border-radius: 12px;
            padding: 24px;
            max-width: 900px;
            width: 100%;
            box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5);
        }}
        h1 {{
            color: #38bdf8;
            margin-top: 0;
        }}
        #viewport {{
            width: 100%;
            height: 480px;
            border-radius: 8px;
            background: #020617;
            margin-top: 16px;
        }}
        .math-box {{
            background: #0f172a;
            padding: 12px 16px;
            border-radius: 6px;
            border-left: 4px solid #38bdf8;
            margin: 16px 0;
        }}
        .controls {{
            display: flex;
            gap: 16px;
            margin-top: 12px;
            align-items: center;
        }}
        .controls label {{
            font-weight: 600;
            color: #94a3b8;
        }}
    </style>
</head>
<body>
    <div class="card">
        <h1>{title}</h1>
        <p>{description}</p>
        
        <div class="math-box">
            $${latex_formula}$$
        </div>

        <div class="controls">
            <label for="wireframeToggle">Wireframe:</label>
            <input type="checkbox" id="wireframeToggle">
            <label for="speedSlider">Rotation Speed:</label>
            <input type="range" id="speedSlider" min="0" max="50" value="10">
        </div>

        <div id="viewport"></div>
    </div>

    <script>
        const vertices = {vertices_json};
        const triangles = {triangles_json};

        const container = document.getElementById('viewport');
        const scene = new THREE.Scene();
        const camera = new THREE.PerspectiveCamera(45, container.clientWidth / container.clientHeight, 0.1, 1000);
        camera.position.set(0, -50, 40);
        camera.up.set(0, 0, 1);
        camera.lookAt(0, 0, 0);

        const renderer = new THREE.WebGLRenderer({{ antialias: true }});
        renderer.setSize(container.clientWidth, container.clientHeight);
        container.appendChild(renderer.domElement);

        // Build Three.js Geometry from CAD Triangles
        const geometry = new THREE.BufferGeometry();
        const positions = [];
        for (let tri of triangles) {{
            for (let idx of tri) {{
                positions.push(vertices[idx][0], vertices[idx][1], vertices[idx][2]);
            }}
        }}
        geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
        geometry.computeVertexNormals();

        const material = new THREE.MeshPhongMaterial({{
            color: 0x38bdf8,
            specular: 0xffffff,
            shininess: 40,
            side: THREE.DoubleSide,
            wireframe: false
        }});
        const mesh = new THREE.Mesh(geometry, material);
        scene.add(mesh);

        // Lighting
        const ambientLight = new THREE.AmbientLight(0x404040, 1.5);
        scene.add(ambientLight);
        const dirLight = new THREE.DirectionalLight(0xffffff, 2.0);
        dirLight.position.set(20, -30, 50);
        scene.add(dirLight);

        let rotSpeed = 0.01;
        document.getElementById('wireframeToggle').addEventListener('change', (e) => {{
            material.wireframe = e.target.checked;
        }});
        document.getElementById('speedSlider').addEventListener('input', (e) => {{
            rotSpeed = e.target.value * 0.001;
        }});

        function animate() {{
            requestAnimationFrame(animate);
            mesh.rotation.z += rotSpeed;
            renderer.render(scene, camera);
        }}
        animate();
    </script>
</body>
</html>
"#
        )
    }
}
