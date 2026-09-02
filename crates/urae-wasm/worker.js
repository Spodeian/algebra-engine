import init, { wasm_process_json, wasm_differentiate, wasm_integrate, wasm_solve, wasm_simplify } from './public/pkg/urae_wasm.js';
import wasmModule from './public/pkg/urae_wasm_bg.wasm';

let wasmInitialized = false;

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);

    // CORS Headers
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
    };

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    // 1. Serve static site assets for Cloudflare Pages / Workers Site immediately if requested
    if (env.ASSETS) {
      const isStaticPath = url.pathname === '/' || 
                           url.pathname === '/index.html' || 
                           url.pathname.startsWith('/pkg/') || 
                           url.pathname.endsWith('.html') || 
                           url.pathname.endsWith('.js') || 
                           url.pathname.endsWith('.wasm') || 
                           url.pathname.endsWith('.css') || 
                           url.pathname.endsWith('.json') || 
                           url.pathname.endsWith('.ico');
      if (isStaticPath) {
        try {
          const assetResp = await env.ASSETS.fetch(request);
          if (assetResp && assetResp.status !== 404) {
            return assetResp;
          }
        } catch (e) {
          console.warn("Asset fetch fallback:", e);
        }
      }
    }

    // 2. Serverless REST & JSON API Endpoints (Lazy WASM Initialization)
    if (!wasmInitialized) {
      try {
        await init(wasmModule);
        wasmInitialized = true;
      } catch (e) {
        console.error("WASM initialization on Worker server failed:", e);
      }
    }

    // Serverless JSON API Endpoint: /api/json
    if (url.pathname === '/api/json' && request.method === 'POST') {
      const reqText = await request.text();
      const respText = wasm_process_json(reqText);
      return new Response(respText, {
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
      });
    }

    // Direct REST Endpoints: /api/differentiate, /api/integrate, /api/solve, /api/simplify
    if (url.pathname === '/api/differentiate') {
      const expr = url.searchParams.get('expr') || 'x^2';
      const wrt = url.searchParams.get('wrt') || 'x';
      const result = wasm_differentiate(expr, wrt);
      return new Response(JSON.stringify({ result }), {
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
      });
    }

    if (url.pathname === '/api/solve') {
      const eq = url.searchParams.get('eq') || 'x^2 - 4 = 0';
      const wrt = url.searchParams.get('wrt') || 'x';
      const result = wasm_solve(eq, wrt);
      return new Response(JSON.stringify({ result }), {
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
      });
    }

    if (url.pathname === '/api/simplify') {
      const expr = url.searchParams.get('expr') || '(x + 0) * 1';
      const result = wasm_simplify(expr);
      return new Response(JSON.stringify({ result }), {
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
      });
    }

    // Fallback static asset handler
    if (env.ASSETS) {
      return env.ASSETS.fetch(request);
    }

    return new Response("URAE Serverless Edge Engine Active", { headers: corsHeaders });
  },
};
