
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

function getErrorMessage(error) {
  if (typeof error === "string") return error;

  return error?.message || "Bilinmeyen hata";
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

async function saveProfile(event) {
  event.preventDefault();
  event.stopImmediatePropagation();

  if (!connected) {
    notify("Önce Holochain ağına bağlanmalısın. Profil yerel tarayıcıya kaydedilmeyecek.");
    return;
  }

  if (busy) return;

  const displayName = $("displayName").value.trim();
  const description = $("description").value.trim();
  const agentType = getSelectedAgentType();

  if (!displayName || displayName.length > 64) {
    notify("Görünen ad 1–64 karakter arasında olmalıdır.");
    return;
  }

  if (description.length > 280) {
    notify("Açıklama en fazla 280 karakter olabilir.");
    return;
  }

  setBusy(true);

  if (saveButton) {
    saveButton.textContent = "Ağa kaydediliyor...";
  }

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

    const profile = {
      schemaVersion: 1,
      displayName,
      agentType,
      description,
      id: encodeHashToBase64(actionHash)
    };

    saveProfileCache(profile);
    showProfile(profile);

    setStatus("Profil Holochain ağına kaydedildi.", true);
    notify("Profil başarıyla Holochain ağına gönderildi.");
  } catch (error) {
    console.error("HoloTrust profile write:", error);

    notify("Profil kaydedilemedi: " + getErrorMessage(error));

    if (getErrorMessage(error).includes("already has a profile")) {
      notify("Bu ajanın zaten profili var. Önce DHT üzerinden profilini oku.");
    }
  } finally {
    setBusy(false);

    if (saveButton) {
      saveButton.textContent = localStorage.getItem(hashKey)
        ? "Profili güncelle"
        : "Profili kaydet";
    }
  }
}

async function lookupPublicProfile() {
  const input = $("publicAgentHash");
  const result = $("publicProfileResult");
  const button = $("publicLookupButton");

  if (!input || !result || !button) return;

  const value = input.value.trim();

  if (!value) {
    result.textContent = "Görüntülemek istediğin ajanın ActionHash bilgisini gir.";
    return;
  }

  if (!connected) {
    result.textContent = "Önce Holochain ağına bağlanmalısın.";
    return;
  }

  button.disabled = true;
  result.textContent = "Profil ağda aranıyor...";

  try {
    const agentPubKey = decodeHashFromBase64(value);

    const record = await callIdentity("get_profile", {
      agent_pub_key: agentPubKey
    });

    if (!record) {
      result.textContent = "Bu ajan için geçerli profil bulunamadı.";
      return;
    }

    const entry = decodeRecordEntry(record);
    const actionHash = getRecordActionHash(record);
    const profile = normalizeProfile(entry, actionHash);

    result.textContent =
      "Görünen ad: " + profile.displayName +
      "\nAjan türü: " + profile.agentType +
      "\nAçıklama: " + (profile.description || "Yok") +
      "\nProfil ActionHash: " + profile.id;
  } catch (error) {
    console.error("HoloTrust public lookup:", error);
    result.textContent = "Profil sorgulanamadı: " + getErrorMessage(error);
  } finally {
    button.disabled = false;
  }
}

// Ajan türü seçimleri
document.querySelectorAll(".type").forEach(button => {
  button.addEventListener("click", () => {
    document.querySelectorAll(".type").forEach(item => {
      item.classList.toggle("active", item === button);
    });
  });
});

// Mevcut HTML bağlantı butonu
connectButton?.addEventListener("click", connect);

// Mevcut HTML profil okuma butonu
readButton?.addEventListener("click", readMyProfile);

// Form olayını capture aşamasında yakala.
// HTML içindeki eski localStorage kayıt mekanizmasının çalışmasını engelle.
form?.addEventListener("submit", saveProfile, true);

// Public profil sorgulama
$("publicLookupButton")?.addEventListener("click", lookupPublicProfile);

// Kimliği kopyala
$("copyButton")?.addEventListener("click", async () => {
  const profileId = localStorage.getItem(hashKey);

  if (!profileId) {
    notify("Kopyalanacak Holochain profil kimliği bulunamadı.");
    return;
  }

  try {
    await navigator.clipboard.writeText(profileId);
    notify("Holochain profil kimliği kopyalandı.");
  } catch (error) {
    notify("Kimlik kopyalanamadı: " + getErrorMessage(error));
  }
});

// Profil düzenleme
$("editButton")?.addEventListener("click", () => {
  const cached = localStorage.getItem(storageKey);

  if (!cached) {
    notify("Düzenlenecek profil bulunamadı.");
    return;
  }

  try {
    const profile = JSON.parse(cached);
    populateForm(profile);
    notify("Profil düzenleme alanına yüklendi.");
  } catch (error) {
    notify("Profil verisi okunamadı: " + getErrorMessage(error));
  }
});

// İlk durum
setStatus("Holochain bağlantısı bekleniyor.");

if (readButton) {
  readButton.disabled = true;
}
