import {
  connectHolochain,
  callIdentity
} from "./holochain.js";

import {
  encodeHashToBase64,
  decodeHashFromBase64
} from "https://esm.sh/@holochain/client@0.21.0";

import { decode } from "https://esm.sh/@msgpack/msgpack@3.1.2";

const $ = id => document.getElementById(id);

const form = $("profileForm");
const connectionStatus = $("connectionStatus");
const connectButton = $("connectButton");
const readButton = $("readButton");
const saveButton = $("saveButton");
const profileCard = $("profileCard");
const toast = $("toast");

// Yerel kayıt yalnızca ÖNBELLEKTİR. Gerçek kaynak Holochain DHT'dir.
const storageKey = "holotrust.profile.v1";
const hashKey = "holotrust.chain.actionHash";

let connection = null;
let connected = false;
let busy = false;

function notify(message) {
  if (!toast) return;
  toast.textContent = message;
  toast.classList.remove("hidden");
}

function getErrorMessage(error) {
  if (typeof error === "string") return error;
  return error?.message || "Bilinmeyen hata";
}

function setStatus(message, active = false) {
  if (!connectionStatus) return;
  connectionStatus.textContent = message;
  connectionStatus.style.borderLeft = active
    ? "4px solid #65e6a0"
    : "4px solid #e8b65b";
}

function setBusy(value) {
  busy = value;
  if (connectButton) {
    connectButton.disabled = value || connected;
  }
  if (readButton) {
    readButton.disabled = value || !connected;
  }
  if (saveButton) {
    saveButton.disabled = value;
  }
}

function getSelectedAgentType() {
  return document.querySelector(".type.active")?.dataset.type || "Human";
}

function decodeRecordEntry(record) {
  const entry = record?.entry;
  if (!entry) {
    throw new Error("Profil kaydında entry alanı bulunamadı.");
  }
  const present = entry.Present;
  if (!present) {
    throw new Error("Profil verisi erişilebilir değil.");
  }
  const bytes = present.entry ?? present;
  if (!(bytes instanceof Uint8Array)) {
    throw new Error("Profil verisi geçerli MessagePack verisi değil.");
  }
  return decode(bytes);
}

function getRecordActionHash(record) {
  const hash = record?.signed_action?.hashed?.hash;
  if (!hash) {
    throw new Error("Profil ActionHash bilgisi bulunamadı.");
  }
  return hash;
}

function normalizeProfile(entry, actionHash) {
  const displayName = entry?.display_name;
  const agentType = entry?.agent_type;
  if (typeof displayName !== "string" || !displayName.trim()) {
    throw new Error("Profil adı geçersiz.");
  }
  if (!["Human", "AI", "Service"].includes(agentType)) {
    throw new Error("Ajan türü tanınmıyor.");
  }
  return {
    schemaVersion: entry.schema_version ?? 1,
    displayName,
    agentType,
    description: entry.description || "",
    id: encodeHashToBase64(actionHash)
  };
}

function saveProfileCache(profile) {
  localStorage.setItem(storageKey, JSON.stringify(profile));
  localStorage.setItem(hashKey, profile.id);
}

function showProfile(profile) {
  if (!profileCard) return;
  $("cardName").textContent = profile.displayName;
  $("cardType").textContent = profile.agentType;
  $("cardDescription").textContent =
    profile.description || "Henüz açıklama eklenmedi.";
  $("profileId").textContent = profile.id;
  profileCard.classList.remove("hidden");
  if (saveButton) {
    saveButton.textContent = "Profili güncelle";
  }
}

function clearProfile() {
  localStorage.removeItem(storageKey);
  localStorage.removeItem(hashKey);
  if (profileCard) {
    profileCard.classList.add("hidden");
  }
  if (saveButton) {
    saveButton.textContent = "Profili kaydet";
  }
}

function populateForm(profile) {
  $("displayName").value = profile.displayName;
  $("description").value = profile.description || "";
  document.querySelectorAll(".type").forEach(button => {
    button.classList.toggle(
      "active",
      button.dataset.type === profile.agentType
    );
  });
}

async function readMyProfile() {
  if (!connected || busy) return;
  setBusy(true);
  if (readButton) {
    readButton.textContent = "DHT okunuyor...";
  }
  try {
    const record = await callIdentity("get_my_profile", null);
    if (!record) {
      clearProfile();
      setStatus("Bağlandı · Henüz profil oluşturulmamış.", true);
      notify("DHT üzerinde henüz profil bulunamadı. Yeni profil oluşturabilirsin.");
      return;
    }
    const entry = decodeRecordEntry(record);
    const actionHash = getRecordActionHash(record);
    const profile = normalizeProfile(entry, actionHash);
    saveProfileCache(profile);
    populateForm(profile);
    showProfile(profile);
    setStatus("Holochain ağına bağlı · Profil DHT'den yüklendi.", true);
    notify("Profil başarıyla Holochain ağından okundu.");
  } catch (error) {
    console.error("HoloTrust profile read:", error);
    notify("Profil okunamadı: " + getErrorMessage(error));
  } finally {
    setBusy(false);
    if (readButton) {
      readButton.textContent = "Profilimi ağdan oku";
    }
  }
}

async function connect() {
  if (busy || connected) return;
  setBusy(true);
  if (connectButton) {
    connectButton.textContent = "Bağlanıyor...";
  }
  setStatus("Holochain bağlantısı kuruluyor...");
  try {
    connection = await connectHolochain();
    connected = true;
    const agentKey = encodeHashToBase64(connection.agentPubKey);
    setStatus(
      "Holochain bağlantısı aktif · " + agentKey.slice(0, 16),
      true
    );
    if (connectButton) {
      connectButton.textContent = "Bağlantı aktif";
    }
    notify("Holochain bağlantısı başarıyla kuruldu.");
    setBusy(false);
    await readMyProfile();
  } catch (error) {
    connection = null;
    connected = false;
    console.error("HoloTrust connection:", error);
    setStatus("Holochain bağlantısı kurulamadı.");
    if (connectButton) {
      connectButton.textContent = "Tekrar bağlan";
    }
    notify("Bağlantı hatası: " + getErrorMessage(error));
  } finally {
    setBusy(false);
  }
}
