import http from "node:http";
import { readFile, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  AdminWebsocket,
  AppWebsocket,
  CellType
} from "@holochain/client";
import { decode, encode } from "@msgpack/msgpack";

const ADMIN_PORT = Number(process.env.HC_ADMIN_PORT ?? 9000);
const APP_PORT = Number(process.env.HC_APP_PORT ?? 9101);
const WEB_PORT = Number(process.env.WEB_PORT ?? 8080);
const HOST = process.env.HOST ?? "127.0.0.1";
const APP_ID = process.env.HC_APP_ID ?? "holotrust";
const ORIGIN = "holotrust-smoke";
const ROLE_NAME = "identity";

const ALLOWED_FUNCTIONS = {
  identity_coordinator: new Set([
    "create_profile",
    "get_my_profile",
    "get_profile",
    "update_profile",
    "create_review",
    "get_reviews",
    "get_reputation_summary"
  ]),
  reputation_coordinator: new Set([
    "create_contextual_review",
    "get_contextual_reviews",
    "get_contextual_reviews_with_provenance",
    "get_contextual_reputation_summary"
  ])
};

const WEB_ROOT = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../web"
);

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".ico": "image/x-icon"
};

async function setup() {
  const adminWs = await AdminWebsocket.connect({
    url: new URL("ws://127.0.0.1:" + ADMIN_PORT),
    wsClientOptions: { origin: ORIGIN }
  });

  try {
    await adminWs.attachAppInterface({
      port: APP_PORT,
      allowed_origins: ORIGIN
    });
  } catch (error) {
    console.log(
      "INFO: app interface may already exist: " +
      (error?.message ?? error)
    );
  }

  const issued = await adminWs.issueAppAuthenticationToken({
    installed_app_id: APP_ID
  });

  const appWs = await AppWebsocket.connect({
    url: new URL("ws://127.0.0.1:" + APP_PORT),
    token: issued.token,
    wsClientOptions: { origin: ORIGIN }
  });

  const appInfo = await appWs.appInfo();
  const cell = appInfo.cell_info[ROLE_NAME]?.[0];

  if (!cell || cell.type !== CellType.Provisioned) {
    throw new Error("Identity cell not found in app " + APP_ID);
  }

  const cellId = cell.value.cell_id;

  await adminWs.authorizeSigningCredentials(cellId);

  return {
    adminWs,
    appWs,
    cellId,
    agentKey: appInfo.agent_pub_key
  };
}

function sendMsgpack(res, status, value) {
  res.writeHead(status, {
    "Content-Type": "application/msgpack",
    "Cache-Control": "no-store"
  });

  res.end(Buffer.from(encode(value)));
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    let size = 0;

    req.on("data", chunk => {
      size += chunk.length;

      if (size > 1_000_000) {
        reject(new Error("Body too large"));
        req.destroy();
        return;
      }

      chunks.push(chunk);
    });

    req.on("end", () => resolve(Buffer.concat(chunks)));
    req.on("error", reject);
  });
}

async function serveStatic(req, res) {
  const url = new URL(req.url, "http://localhost");
  let pathname = decodeURIComponent(url.pathname);

  if (pathname.endsWith("/")) {
    pathname += "index.html";
  }

  const filePath = path.normalize(path.join(WEB_ROOT, pathname));

  if (!filePath.startsWith(WEB_ROOT + path.sep)) {
    res.writeHead(403);
    res.end("Forbidden");
    return;
  }

  try {
    const info = await stat(filePath);

    if (!info.isFile()) {
      throw new Error("Not a file");
    }

    const data = await readFile(filePath);

    res.writeHead(200, {
      "Content-Type":
        MIME[path.extname(filePath)] ?? "application/octet-stream",
      "Cache-Control": "no-store"
    });

    res.end(data);
  } catch {
    res.writeHead(404);
    res.end("Not found");
  }
}

async function main() {
  const ctx = await setup();

  console.log("INFO: connected to conductor, app " + APP_ID);

  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url, "http://localhost");

      if (req.method === "GET" && url.pathname === "/hc/info") {
        sendMsgpack(res, 200, {
          agent_pub_key: ctx.agentKey,
          app_id: APP_ID
        });
        return;
      }

      if (
        req.method === "POST" &&
        url.pathname.startsWith("/hc/call/")
      ) {
        const origin = req.headers.origin;

        if (origin && new URL(origin).host !== req.headers.host) {
          sendMsgpack(res, 403, {
            error: "Cross-origin request blocked"
          });
          return;
        }

        const parts = url.pathname
          .slice("/hc/call/".length)
          .split("/");

        if (parts.length !== 2) {
          sendMsgpack(res, 400, {
            error: "Expected zome name and function name"
          });
          return;
        }

        const [zomeName, fnName] = parts.map(decodeURIComponent);
        const allowed = ALLOWED_FUNCTIONS[zomeName];

        if (!allowed || !allowed.has(fnName)) {
          sendMsgpack(res, 403, {
            error: "Function not allowed"
          });
          return;
        }

        const raw = await readBody(req);
        const payload =
          raw.length === 0 ? null : decode(new Uint8Array(raw));

        try {
          const result = await ctx.appWs.callZome({
            cell_id: ctx.cellId,
            zome_name: zomeName,
            fn_name: fnName,
            provenance: ctx.agentKey,
            payload
          });

          sendMsgpack(res, 200, {
            result: result ?? null
          });
        } catch (error) {
          sendMsgpack(res, 500, {
            error: String(error?.message ?? error)
          });
        }

        return;
      }

      if (req.method === "GET") {
        await serveStatic(req, res);
        return;
      }

      res.writeHead(405);
      res.end("Method not allowed");
    } catch (error) {
      console.error(String(error?.stack ?? error));
      res.writeHead(500);
      res.end("Server error");
    }
  });

  server.listen(WEB_PORT, HOST, () => {
    console.log(
      "INFO: HoloTrust dev server on http://" +
      HOST +
      ":" +
      WEB_PORT
    );
  });
}

main().catch(error => {
  console.error(String(error?.stack ?? error));
  process.exit(1);
});
