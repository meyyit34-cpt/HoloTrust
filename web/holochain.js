import { AppWebsocket } from "https://esm.sh/@holochain/client@0.21.0";

let connectionPromise = null;

const ROLE_NAME = "identity";
const ZOME_NAME = "identity_coordinator";

const ALLOWED_FUNCTIONS = new Set([
  "create_profile",
  "get_my_profile",
  "get_profile",
  "update_profile"
]);

export async function connectHolochain() {
  if (!connectionPromise) {
    connectionPromise = (async () => {
      const client = await AppWebsocket.connect();
      const appInfo = await client.appInfo();

      const cells = appInfo?.cell_info?.[ROLE_NAME];

      if (!cells?.length) {
        throw new Error("Identity hücresi bulunamadı.");
      }

      const cell = cells.find(item => item.value?.cell_id);

      if (!cell) {
        throw new Error("Identity hücresi henüz hazır değil.");
      }

      if (!appInfo.agent_pub_key) {
        throw new Error("Agent public key alınamadı.");
      }

      return {
        client,
        appInfo,
        cellId: cell.value.cell_id,
        agentPubKey: appInfo.agent_pub_key
      };
    })().catch(error => {
      connectionPromise = null;
      throw error;
    });
  }

  return connectionPromise;
}

export async function callIdentity(fnName, payload = null) {
  if (!ALLOWED_FUNCTIONS.has(fnName)) {
    throw new Error("İzin verilmeyen zome fonksiyonu.");
  }

  const connection = await connectHolochain();

  return connection.client.callZome({
    cell_id: connection.cellId,
    zome_name: ZOME_NAME,
    fn_name: fnName,
    provenance: connection.agentPubKey,
    payload
  });
}

export async function disconnectHolochain() {
  if (!connectionPromise) return;

  const connection = await connectionPromise;
  await connection.client.client.close();
  connectionPromise = null;
}
