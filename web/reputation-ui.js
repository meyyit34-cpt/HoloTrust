import {
  connectHolochain,
  callReputation
} from "./holochain.js";

import {
  encodeHashToBase64,
  decodeHashFromBase64
} from "https://esm.sh/@holochain/client@0.21.1";

const $ = id => document.getElementById(id);

const DOMAINS = {
  commerce: "Ticaret",
  professional: "Profesyonellik",
  hospitality: "Misafirperverlik",
  technical: "Teknik yeterlilik",
  community: "Topluluk",
  reliability: "Güvenilirlik"
};

const STATUS_LABELS = {
  Active: "Aktif",
  Revoked: "İptal edilmiş",
  Expired: "Süresi dolmuş",
  Superseded: "Yeni kayıtla değiştirilmiş",
  Missing: "Kayıt bulunamadı",
  Invalid: "Geçersiz kayıt",
  Unavailable: "Durum doğrulanamıyor"
};

let connection = null;
let busy = false;

function setMessage(message, isError = false) {
  const element = $("repMessage");
  if (!element) return;

  element.textContent = message;
  element.classList.remove("hidden");
  element.style.borderLeft = isError
    ? "4px solid #e87979"
    : "4px solid #66e3c4";
}

function setConnectionStatus(message, active = false) {
  const element = $("repConnectionStatus");
  if (!element) return;

  element.textContent = message;
  element.style.borderLeft = active
    ? "4px solid #66e3c4"
    : "4px solid #e8b65b";
}

function setBusy(value) {
  busy = value;

  const loadButton = $("repLoadButton");
  const connectButton = $("repConnectButton");
  const submitButton = $("repSubmitButton");

  if (loadButton) loadButton.disabled = value;
  if (connectButton) connectButton.disabled = value;
  if (submitButton) submitButton.disabled = value;
}

async function ensureConnection() {
  if (!connection) {
    connection = await connectHolochain();
  }

  if (!(connection.agentPubKey instanceof Uint8Array)) {
    throw new Error("Agent Public Key alınamadı.");
  }

  return connection;
}

function encodeHash(hash) {
  try {
    return encodeHashToBase64(hash);
  } catch {
    return "Hash bilgisi okunamadı.";
  }
}

function addText(parent, tag, className, value) {
  const element = document.createElement(tag);

  if (className) element.className = className;

  element.textContent = value == null ? "" : String(value);
  parent.appendChild(element);

  return element;
}

function evidenceStatusLabel(status) {
  return STATUS_LABELS[status] || "Bilinmeyen durum";
}

async function connectReputation() {
  if (busy) return;

  setBusy(true);
  setConnectionStatus("Holochain bağlantısı kuruluyor...");

  try {
    const activeConnection = await ensureConnection();
    const ownKey = encodeHash(activeConnection.agentPubKey);

    $("repSubjectAgent").value = ownKey;

    setConnectionStatus(
      "Holochain bağlantısı aktif · " + ownKey.slice(0, 18) + "...",
      true
    );

    setMessage("Holochain bağlantısı başarıyla kuruldu.");
  } catch (error) {
    connection = null;
    setConnectionStatus("Bağlantı kurulamadı.");
    setMessage(error?.message || String(error), true);
  } finally {
    setBusy(false);
  }
}

function renderSummary(summaries) {
  const container = $("repSummary");
  container.replaceChildren();

  for (const domain of Object.keys(DOMAINS)) {
    const summary = summaries.find(item => item.domain === domain);

    const card = document.createElement("div");
    card.className = "rep-card";

    addText(card, "strong", "", DOMAINS[domain]);

    const count = Number(summary?.review_count ?? 0);
    const average = Number(summary?.average_rating ?? 0);

    addText(
      card,
      "div",
      "rep-score",
      count === 0 ? "—" : average.toFixed(2) + " / 5"
    );

    addText(card, "div", "rep-muted", count + " değerlendirme");

    container.appendChild(card);
  }
}

function renderEvidence(item, proof) {
  const evidenceItem = document.createElement("div");
  evidenceItem.className = "rep-domain";

  addText(
    evidenceItem,
    "strong",
    "",
    "Kanıt durumu: " + evidenceStatusLabel(proof.status)
  );

  addText(
    evidenceItem,
    "div",
    "rep-hash",
    "Attestation ActionHash: " + encodeHash(proof.action_hash)
  );

  if (proof.issuer) {
    addText(
      evidenceItem,
      "div",
      "rep-hash",
      "Düzenleyen ajan: " + encodeHash(proof.issuer)
    );
  }

  if (proof.issued_at !== null && proof.issued_at !== undefined) {
    addText(
      evidenceItem,
      "div",
      "rep-muted",
      "Oluşturulma zamanı: " + String(proof.issued_at)
    );
  }

  if (proof.attestation) {
    addText(
      evidenceItem,
      "div",
      "",
      "İddia türü: " + String(proof.attestation.claim_type ?? "Bilinmiyor")
    );

    addText(
      evidenceItem,
      "p",
      "",
      "İddia: " + String(proof.attestation.claim ?? "Açıklama yok")
    );

    addText(
      evidenceItem,
      "div",
      "rep-hash",
      "İlgili ajan: " + encodeHash(proof.attestation.subject)
    );
  } else {
    addText(
      evidenceItem,
      "p",
      "rep-muted",
      "Bu kanıt için attestation içeriği alınamadı."
    );
  }

  item.appendChild(evidenceItem);
}

function renderReviews(records) {
  const container = $("repReviews");
  container.replaceChildren();

  if (!Array.isArray(records) || records.length === 0) {
    addText(
      container,
      "p",
      "muted",
      "Bu alanda henüz değerlendirme bulunmuyor."
    );
    return;
  }

  for (const record of records) {
    if (!record?.review) continue;

    const review = record.review;
    const item = document.createElement("div");
    item.className = "rep-review";

    addText(
      item,
      "strong",
      "",
      DOMAINS[review.domain] + " · ★ " + review.rating + " / 5"
    );

    if (review.comment) {
      addText(item, "p", "", review.comment);
    }

    addText(
      item,
      "div",
      "rep-muted",
      "Değerlendiren ajan: " + encodeHash(record.author)
    );

    addText(
      item,
      "div",
      "rep-hash",
      "Değerlendirme ActionHash: " + encodeHash(record.action_hash)
    );

    addText(
      item,
      "div",
      "rep-muted",
      "Kayıt zamanı: " + String(record.timestamp ?? "Bilinmiyor")
    );

    const evidence = Array.isArray(record.evidence)
      ? record.evidence
      : [];

    addText(
      item,
      "div",
      "rep-muted",
      "Kanıt sayısı: " + evidence.length
    );

    for (const proof of evidence) {
      renderEvidence(item, proof);
    }

    container.appendChild(item);
  }
}

async function loadReputation() {
  if (busy) return;

  const subjectValue = $("repSubjectAgent").value.trim();

  if (!subjectValue) {
    setMessage("Önce incelenecek ajanın Agent Public Key bilgisini gir.", true);
    return;
  }

  setBusy(true);

  $("repSummary").textContent = "İtibar özeti yükleniyor...";
  $("repReviews").textContent = "Değerlendirmeler yükleniyor...";

  try {
    await ensureConnection();

    const subjectAgent = decodeHashFromBase64(subjectValue);
    const domain = $("repDomain").value;

    const [summaries, records] = await Promise.all([
      callReputation("get_contextual_reputation_summary", {
        subject_agent: subjectAgent,
        domain: null
      }),
      callReputation("get_contextual_reviews_with_provenance", {
        subject_agent: subjectAgent,
        domain
      })
    ]);

    renderSummary(summaries);
    renderReviews(records);

    setConnectionStatus("İtibar verileri Holochain ağından yüklendi.", true);
    setMessage("İtibar bilgileri başarıyla yüklendi.");
  } catch (error) {
    $("repSummary").textContent = "İtibar özeti alınamadı.";
    $("repReviews").textContent = "Değerlendirmeler alınamadı.";

    setMessage(error?.message || String(error), true);
  } finally {
    setBusy(false);
  }
}

function parseEvidenceHashes(value) {
  const values = value
    .split(/[\s,;]+/)
    .map(item => item.trim())
    .filter(Boolean);

  if (values.length < 1 || values.length > 5) {
    throw new Error("En az 1, en fazla 5 kanıt ActionHash girmelisin.");
  }

  const unique = new Set(values);

  if (unique.size !== values.length) {
    throw new Error("Aynı kanıt ActionHash değerini birden fazla giremezsin.");
  }

  return values.map(value => decodeHashFromBase64(value));
}

async function createReview(event) {
  event.preventDefault();

  if (busy) return;

  setBusy(true);

  try {
    const activeConnection = await ensureConnection();

    const subjectValue = $("repSubjectAgent").value.trim();

    if (!subjectValue) {
      throw new Error("İncelenecek ajanın Agent Public Key bilgisi gerekli.");
    }

    const subjectAgent = decodeHashFromBase64(subjectValue);
    const ownAgent = encodeHashToBase64(activeConnection.agentPubKey);

    if (ownAgent === subjectValue) {
      throw new Error("Kendi ajanına değerlendirme gönderemezsin.");
    }

    const rating = Number($("repRating").value);
    const comment = $("repComment").value.trim();
    const evidenceAttestations = parseEvidenceHashes(
      $("repEvidence").value
    );

    if (!Number.isInteger(rating) || rating < 1 || rating > 5) {
      throw new Error("Puan 1 ile 5 arasında olmalıdır.");
    }

    if (comment.length > 500) {
      throw new Error("Açıklama en fazla 500 karakter olabilir.");
    }

    const result = await callReputation("create_contextual_review", {
      subject_agent: subjectAgent,
      domain: $("repDomain").value,
      rating,
      comment: comment || null,
      evidence_attestations: evidenceAttestations
    });

    $("repComment").value = "";
    $("repEvidence").value = "";

    setMessage(
      "Değerlendirme Holochain ağına gönderildi. ActionHash: " +
      encodeHash(result)
    );

    await loadReputation();
  } catch (error) {
    setMessage(error?.message || String(error), true);
  } finally {
    setBusy(false);
  }
}

$("repConnectButton")?.addEventListener("click", connectReputation);
$("repLoadButton")?.addEventListener("click", loadReputation);
$("repCreateForm")?.addEventListener("submit", createReview);

$("repDomain")?.addEventListener("change", () => {
  if ($("repSubjectAgent").value.trim()) {
    loadReputation();
  }
});

setConnectionStatus("Holochain bağlantısı bekleniyor.");
