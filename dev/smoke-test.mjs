import {
  AdminWebsocket,
  AppWebsocket,
  CellType,
  encodeHashToBase64
} from "@holochain/client";
import { decode } from "@msgpack/msgpack";

const ADMIN_PORT = 9000;
const APP_PORT = 9101;
const APP_ID = "holotrust";
const ORIGIN = "holotrust-smoke";
const ROLE_NAME = "identity";
const ZOME_NAME = "identity_coordinator";

function check(condition, message) {
  if (!condition) {
    throw new Error("SMOKE FAIL: " + message);
  }
  console.log("SMOKE PASS: " + message);
}

function decodeProfile(record) {
  const present = record?.entry?.Present;
  if (!present) {
    throw new Error(
      "SMOKE FAIL: record.entry.Present missing. entry keys: " +
        JSON.stringify(Object.keys(record?.entry ?? {}))
    );
  }
  const bytes = present.entry ?? present;
  return decode(bytes);
}

function recordHash(record) {
  return encodeHashToBase64(record.signed_action.hashed.hash);
}

async function main() {
  const adminWs = await AdminWebsocket.connect({
    url: new URL("ws://127.0.0.1:" + ADMIN_PORT)
  });
  console.log("SMOKE INFO: admin websocket connected");

  await adminWs.attachAppInterface({
    port: APP_PORT,
    allowed_origins: ORIGIN
  });

  const issued = await adminWs.issueAppAuthenticationToken({
    installed_app_id: APP_ID
  });

  const appWs = await AppWebsocket.connect({
    url: new URL("ws://127.0.0.1:" + APP_PORT),
    token: issued.token,
    wsClientOptions: { origin: ORIGIN }
  });
  console.log("SMOKE INFO: app websocket connected");

  const appInfo = await appWs.appInfo();
  const cell = appInfo.cell_info[ROLE_NAME]?.[0];
  check(
    cell && cell.type === CellType.Provisioned,
    "identity cell is provisioned"
  );

  const cellId = cell.value.cell_id;
  await adminWs.authorizeSigningCredentials(cellId);

  const agentKey = appInfo.agent_pub_key;
  check(
    encodeHashToBase64(agentKey).startsWith("uhCAk"),
    "agent public key encodes with uhCAk prefix"
  );

  const call = (fn_name, payload) =>
    appWs.callZome({
      cell_id: cellId,
      zome_name: ZOME_NAME,
      fn_name,
      provenance: agentKey,
      payload
    });

  const before = await call("get_my_profile", null);
  check(before === null || before === undefined, "no profile before creation");

  const created = await call("create_profile", {
    display_name: "Smoke Agent",
    agent_type: "AI",
    description: "ci smoke test"
  });
  check(created instanceof Uint8Array, "create_profile returns an action hash");

  const mine = await call("get_my_profile", null);
  check(mine !== null && mine !== undefined, "profile found after creation");

  const decoded = decodeProfile(mine);
  console.log("SMOKE INFO: decoded profile = " + JSON.stringify(decoded));
  check(decoded.display_name === "Smoke Agent", "display_name decodes");
  check(decoded.agent_type === "AI", "agent_type AI serializes as the string AI");
  check(
    recordHash(mine) === encodeHashToBase64(created),
    "record action hash matches created hash"
  );

  const updated = await call("update_profile", {
    action_hash: created,
    display_name: "Smoke Agent Updated",
    agent_type: "AI",
    description: null
  });
  check(updated instanceof Uint8Array, "update_profile returns an action hash");

  let staleRejected = false;
  try {
    await call("update_profile", {
      action_hash: created,
      display_name: "Stale",
      agent_type: "AI",
      description: null
    });
  } catch (error) {
    staleRejected = true;
    console.log("SMOKE INFO: stale update error = " + (error?.message ?? error));
  }
  check(staleRejected, "update from a stale version is rejected");

  const afterUpdate = await call("get_my_profile", null);
  check(
    recordHash(afterUpdate) === encodeHashToBase64(updated),
    "get_my_profile returns the latest version"
  );
  check(
    decodeProfile(afterUpdate).display_name === "Smoke Agent Updated",
    "updated display_name decodes"
  );

  const lookedUp = await call("get_profile", { agent_pub_key: agentKey });
  check(
    lookedUp !== null && recordHash(lookedUp) === encodeHashToBase64(updated),
    "get_profile by agent public key returns the latest version"
  );

  await appWs.client.close();
  await adminWs.client.close();
  console.log("SMOKE DONE: all checks passed");
  process.exit(0);
}

main().catch(error => {
  console.error(String(error?.stack ?? error));
  process.exit(1);
});
