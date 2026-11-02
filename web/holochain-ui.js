import {
  connectHolochain,
  callIdentity
} from "./holochain.js";

import {
  encodeHashToBase64,
  decodeHashFromBase64
} from "https://esm.sh/@holochain/client@0.21.0";

const $ = id => document.getElementById(id);

const status = document.querySelector(".status");
const form = $("profileForm");
const storageKey = "holotrust.profile.v1";
const actionHashKey = "holotrust.chain.actionHash";

let connected = false;
let busy = false;

const connectButton = document.createElement("button");
connectButton.type = "button";
connectButton.textContent = "Holochain'a bağlan";
connectButton.className = "secondary";
connectButton.style.marginLeft = "10px";
status.appendChild(connectButton);

function setStatus(message, isConnected = false) {
  status.querySelector(".dot").style.background =
    isConnected ? "#65e6a0" : "#e8b65b";

  status.lastChild.textContent = message;
}

function notify(message) {
  const toast = $("toast");
  toast.textContent = message;
  toast.style.display = "block";
  setTimeout(() => toast.style.display = "none", 3000);
}

connectButton.addEventListener("click", async () => {
  if (busy) return;

  busy = true;
  connectButton.disabled = true;
  connectButton.textContent = "Bağlanıyor...";

  try {
    const connection = await connectHolochain();

    connected = true;

    setStatus(
      "Holochain bağlı · " +
      encodeHashToBase64(connection.agentPubKey).slice(0, 12),
      true
    );

    connectButton.textContent = "Bağlantı aktif";
    notify("Holochain bağlantısı kuruldu.");
  } catch (error) {
    connected = false;
    connectButton.disabled = false;
    connectButton.textContent = "Tekrar bağlan";
    setStatus("Holochain bağlantısı kurulamadı");
    notify(error.message || "Bağlantı hatası.");
  } finally {
    busy = false;
  }
});

form.addEventListener("submit", async event => {
  if (!connected) return;

  event.preventDefault();
  event.stopImmediatePropagation();

  if (busy) return;

  const displayName = $("displayName").value.trim();
  const description = $("description").value.trim();

  if (!displayName) {
    notify("Görünen ad zorunludur.");
    return;
  }

  busy = true;
  $("saveButton").disabled = true;
  $("saveButton").textContent = "Ağa kaydediliyor...";

  try {
    const existingHash = localStorage.getItem(actionHashKey);

    let result;

    if (existingHash) {
      result = await callIdentity("update_profile", {
        action_hash: decodeHashFromBase64(existingHash),
        display_name: displayName,
        agent_type: document.querySelector(".type.active").dataset.type,
        description: description || null
      });
    } else {
      result = await callIdentity("create_profile", {
        display_name: displayName,
        agent_type: document.querySelector(".type.active").dataset.type,
        description: description || null
      });
    }

    const actionHash = encodeHashToBase64(result);

    localStorage.setItem(actionHashKey, actionHash);

    const profile = {
      schemaVersion: 1,
      displayName,
      agentType: document.querySelector(".type.active").dataset.type,
      description,
      localId: actionHash,
      updatedAt: new Date().toISOString()
    };

    localStorage.setItem(storageKey, JSON.stringify(profile));

    notify("Profil Holochain ağına gönderildi.");
    location.reload();

  } catch (error) {
    console.error("HoloTrust zome error:", error);
    notify("Kayıt başarısız: " + (error.message || "Bilinmeyen hata"));
  } finally {
    busy = false;
    $("saveButton").disabled = false;
    $("saveButton").textContent = "Profili kaydet";
  }
}, true);
