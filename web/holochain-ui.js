import {
  connectHolochain,
  callIdentity
} from "./holochain.js";

import {
  encodeHashToBase64,
  decodeHashFromBase64
} from "https://esm.sh/@holochain/client@0.21.0";

const $ = id => document.getElementById(id);

const form = $("profileForm");
const status = document.querySelector(".status");

const storageKey = "holotrust.profile.v1";
const hashKey = "holotrust.chain.actionHash";

let connected = false;
let busy = false;

const statusText = document.createElement("span");
statusText.textContent = "Holochain bağlantısı bekleniyor";
status.appendChild(statusText);

const connectButton = document.createElement("button");
connectButton.type = "button";
connectButton.className = "secondary";
connectButton.textContent = "Bağlan";
status.appendChild(connectButton);

const readButton = document.createElement("button");
readButton.type = "button";
readButton.className = "secondary";
readButton.textContent = "Ağdan profilimi oku";
readButton.disabled = true;
readButton.style.marginLeft = "8px";
status.appendChild(readButton);

function notify(message) {
  const toast = $("toast");
  toast.textContent = message;
  toast.style.display = "block";
  setTimeout(() => toast.style.display = "none", 3000);
}

function setStatus(message, active = false) {
  statusText.textContent = message;
  status.querySelector(".dot").style.background =
    active ? "#65e6a0" : "#e8b65b";
}

function selectedAgentType() {
  return document.querySelector(".type.active")?.dataset.type || "Human";
}

function showProfile(profile) {
  const types = {
    Human: { label: "İnsan", icon: "♙" },
    AI: { label: "Yapay zekâ ajanı", icon: "✳" },
    Service: { label: "Dijital servis", icon: "◇" }
  };

  const type = types[profile.agentType] || types.Human;

  $("emptyState").hidden = true;
  $("profileCard").hidden = false;

  $("cardName").textContent = profile.displayName;
  $("cardType").textContent = type.label;
  $("avatar").textContent = type.icon;
  $("cardDescription").textContent =
    profile.description || "Henüz açıklama eklenmedi.";

  $("profileId").textContent = profile.localId;
  $("saveButton").textContent = "Profili güncelle";
}

connectButton.addEventListener("click", async () => {
  if (busy || connected) return;

  busy = true;
  connectButton.disabled = true;
  connectButton.textContent = "Bağlanıyor...";

  try {
    const connection = await connectHolochain();

    connected = true;

    const agentKey = encodeHashToBase64(
      connection.agentPubKey
    );

    setStatus("Bağlandı · " + agentKey.slice(0, 12), true);

    connectButton.textContent = "Bağlantı aktif";
    readButton.disabled = false;

    notify("Holochain bağlantısı kuruldu.");

  } catch (error) {
    connected = false;

    setStatus("Bağlantı kurulamadı");
    connectButton.disabled = false;
    connectButton.textContent = "Tekrar dene";

    notify(error.message || "Bağlantı hatası.");

  } finally {
    busy = false;
  }
});

readButton.addEventListener("click", async () => {
  if (!connected || busy) return;

  busy = true;
  readButton.disabled = true;
  readButton.textContent = "DHT okunuyor...";

  try {
    const record = await callIdentity("get_my_profile", null);

    if (!record) {
      notify("DHT üzerinde henüz profil bulunamadı.");
      return;
    }

    const hash = record.signed_action?.hashed?.hash;

    if (hash) {
      const encodedHash = encodeHashToBase64(hash);
      localStorage.setItem(hashKey, encodedHash);

      const saved = localStorage.getItem(storageKey);

      if (saved) {
        const profile = JSON.parse(saved);
        profile.localId = encodedHash;
        localStorage.setItem(storageKey, JSON.stringify(profile));
        showProfile(profile);
      }
    }

    setStatus("DHT profil kaydı bulundu", true);
    notify("Profil kaydı Holochain DHT üzerinden bulundu.");

  } catch (error) {
    console.error("DHT read error:", error);
    notify("Profil okunamadı: " + (error.message || "Bilinmeyen hata"));

  } finally {
    busy = false;
    readButton.disabled = !connected;
    readButton.textContent = "Ağdan profilimi oku";
  }
});

form.addEventListener("submit", async event => {
  if (!connected) return;

  event.preventDefault();
  event.stopImmediatePropagation();

  if (busy) return;

  const displayName = $("displayName").value.trim();
  const description = $("description").value.trim();
  const agentType = selectedAgentType();

  if (!displayName || displayName.length > 64) {
    notify("Görünen ad 1–64 karakter olmalıdır.");
    return;
  }

  if (description.length > 280) {
    notify("Açıklama en fazla 280 karakter olabilir.");
    return;
  }

  busy = true;
  $("saveButton").disabled = true;
  $("saveButton").textContent = "Ağa kaydediliyor...";

  try {
    const existingHash = localStorage.getItem(hashKey);

    let actionHash;

    if (existingHash) {
      actionHash = await callIdentity("update_profile", {
        action_hash: decodeHashFromBase64(existingHash),
        display_name: displayName,
        agent_type: agentType,
        description: description || null
      });
    } else {
      actionHash = await callIdentity("create_profile", {
        display_name: displayName,
        agent_type: agentType,
        description: description || null
      });
    }

    const encodedHash = encodeHashToBase64(actionHash);

    const profile = {
      schemaVersion: 1,
      displayName,
      agentType,
      description,
      localId: encodedHash,
      updatedAt: new Date().toISOString()
    };

    localStorage.setItem(hashKey, encodedHash);
    localStorage.setItem(storageKey, JSON.stringify(profile));

    showProfile(profile);

    notify("Profil Holochain'a gönderildi.");

  } catch (error) {
    console.error("HoloTrust write error:", error);
    notify("Kayıt başarısız: " + (error.message || "Bilinmeyen hata"));

  } finally {
    busy = false;
    $("saveButton").disabled = false;
    $("saveButton").textContent = "Profili kaydet";
  }
}, true);
