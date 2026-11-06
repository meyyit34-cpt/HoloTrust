import { callReputation } from "./holochain.js";

import {
  decodeHashFromBase64,
  encodeHashToBase64
} from "https://esm.sh/@holochain/client@0.21.0";

import { decode } from "https://esm.sh/@msgpack/msgpack@3.1.2";

const DOMAINS = {
  Commerce: "Ticaret",
  Professional: "Profesyonellik",
  Hospitality: "Misafirperverlik",
  Technical: "Teknik Yetkinlik",
  Community: "Topluluk",
  Reliability: "Güvenilirlik"
};

const style = document.createElement("style");
style.textContent = `
  #reputationPanel {
    margin: 28px auto;
    max-width: 900px;
    padding: 24px;
    color: #e8edf7;
    background: #111827;
    border: 1px solid #293449;
    border-radius: 16px;
    font-family: inherit;
  }

  #reputationPanel input,
  #reputationPanel textarea,
  #reputationPanel select {
    display: block;
    box-sizing: border-box;
    width: 100%;
    margin: 8px 0 14px;
    padding: 12px;
    color: #f8fafc;
    background: #0b1220;
    border: 1px solid #34445d;
    border-radius: 8px;
    font: inherit;
  }

  #reputationPanel button {
    padding: 11px 16px;
    margin: 8px 6px 4px 0;
    color: #07111f;
    background: #66e3c4;
    border: 0;
    border-radius: 8px;
    font-weight: 700;
    cursor: pointer;
  }

  #reputationPanel button:disabled {
    opacity: .5;
    cursor: not-allowed;
  }

  #reputationPanel .rep-secondary {
    color: #e8edf7;
    background: #263449;
    border: 1px solid #40516b;
  }

  #reputationPanel .rep-result {
    margin-top: 14px;
    padding: 14px;
    background: #0b1220;
    border: 1px solid #293449;
    border-radius: 8px;
    overflow-wrap: anywhere;
    white-space: pre-wrap;
  }

  #reputationPanel label {
    display: block;
    margin-top: 14px;
    color: #b9c5d8;
  }

  #reputationPanel .rep-muted {
    color: #9aa9bd;
    font-size: 13px;
    line-height: 1.6;
  }

  #reputationPanel .rep-domain {
    margin: 8px 0;
    padding: 12px;
    background: #172235;
    border: 1px solid #34445d;
    border-radius: 8px;
  }

  #reputationPanel .rep-review {
    margin: 10px 0;
    padding: 13px;
    background: #172235;
    border: 1px solid #34445d;
    border-radius: 8px;
  }

  #reputationPanel .rep-hash {
    font-size: 12px;
    color: #9aa9bd;
    overflow-wrap: anywhere;
  }

  #reputationPanel hr {
    border: 0;
    border-top: 1px solid #293449;
    margin: 24px 0;
  }
`;
document.head.appendChild(style);

const panel = document.createElement("section");
panel.id = "reputationPanel";

panel.innerHTML = `
  <h2>HoloTrust Contextual Reputation</h2>

  <p class="rep-muted">
    İtibar tek bir genel puanla değil, altı ayrı alanda değerlendirilir.
    Her değerlendirme en az bir doğrulanabilir attestation kaydına dayanır.
  </p>

  <label for="repAgentKey">Hedef Agent Public Key</label>
  <input
    id="repAgentKey"
    autocomplete="off"
    spellcheck="false"
    placeholder="uhCAk..."
  >

  <button id="repLookup">İtibar alanlarını sorgula</button>
  <button id="repShowReviews" class="rep-secondary">
    Değerlendirmeleri listele
  </button>

  <div id="repSummary" class="rep-result" aria-live="polite">
    Sorgulama bekleniyor.
  </div>

  <div id="repReviews" class="rep-result" aria-live="polite">
    Değerlendirmeler burada gösterilecek.
  </div>

  <hr>

  <h3>Yeni değerlendirme yayınla</h3>

  <p class="rep-muted">
    Değerlendirme yapabilmek için hedef ajana ait en az bir aktif
    attestation kaydının ActionHash bilgisi gereklidir.
    En fazla 5 kayıt ekleyebilirsin.
  </p>

  <label for="repDomain">İtibar alanı</label>
  <select id="repDomain">
    <option value="Commerce">Ticaret</option>
    <option value="Professional">Profesyonellik</option>
    <option value="Hospitality">Misafirperverlik</option>
    <option value="Technical">Teknik Yetkinlik</option>
    <option value="Community">Topluluk</option>
    <option value="Reliability">Güvenilirlik</option>
  </select>

  <label for="repRating">Puan</label>
  <select id="repRating">
    <option value="5">5 — Çok iyi</option>
    <option value="4">4 — İyi</option>
    <option value="3">3 — Orta</option>
    <option value="2">2 — Zayıf</option>
    <option value="1">1 — Çok kötü</option>
  </select>

  <label for="repEvidence">
    Attestation ActionHash kayıtları
  </label>
  <textarea
    id="repEvidence"
    rows="4"
    placeholder="Her ActionHash değerini ayrı satıra yapıştır..."
  ></textarea>

  <p class="rep-muted">
    Agent Public Key değil, attestation kaydının ActionHash değeri
    girilmelidir.
  </p>

  <label for="repComment">
    Yorum (isteğe bağlı, en fazla 500 karakter)
  </label>
  <textarea
    id="repComment"
    maxlength="500"
    rows="3"
    placeholder="Deneyimini açıkla..."
  ></textarea>

  <button id="repSubmit">Değerlendirmeyi yayınla</button>

  <div id="repStatus" class="rep-result" aria-live="polite">
    Değerlendirme bekleniyor.
  </div>
`;

document.body.appendChild(panel);

const byId = id => document.getElementById(id);

function setStatus(id, message) {
  byId(id).textContent = message;
}

function friendlyError(error) {
  const message = String(error?.message ?? error ?? "");

  if (message.includes("already reviewed")) {
    return "Bu ajanı bu alanda zaten değerlendirdin.";
  }

  if (
    message.includes("cannot review your own") ||
    message.includes("review your own")
  ) {
    return "Kendi ajanını değerlendiremezsin.";
  }

  if (message.includes("At least one evidence")) {
    return "En az bir attestation ActionHash girmelisin.";
  }

  if (message.includes("not active")) {
    return "Eklediğin kanıtlardan biri aktif değil. Attestation durumunu kontrol et.";
  }

  if (message.includes("must belong to the reviewed subject")) {
    return "Attestation kaydı değerlendirdiğin ajana ait değil.";
  }

  if (message.includes("Rating must be")) {
    return "Puan 1 ile 5 arasında olmalıdır.";
  }

  if (message.includes("comment cannot exceed")) {
    return "Yorum en fazla 500 karakter olabilir.";
  }

  return message || "İşlem başarısız.";
}

function readAgentKey() {
  const value = byId("repAgentKey").value.trim();

  if (!value) {
    throw new Error("Hedef Agent Public Key gerekli.");
  }

  if (!value.startsWith("uhCAk")) {
    throw new Error(
      "Agent Public Key 'uhCAk' ile başlamalıdır."
    );
  }

  try {
    return decodeHashFromBase64(value);
  } catch {
    throw new Error("Agent Public Key geçerli formatta değil.");
  }
}

function readEvidenceHashes() {
  const values = byId("repEvidence").value
    .split(/[\n,;]+/)
    .map(value => value.trim())
    .filter(Boolean);

  if (values.length < 1 || values.length > 5) {
    throw new Error(
      "Bir ile beş arasında attestation ActionHash girmelisin."
    );
  }

  const unique = new Set(values);

  if (unique.size !== values.length) {
    throw new Error("Aynı ActionHash değerini birden fazla ekleme.");
  }

  return values.map(value => {
    try {
      return decodeHashFromBase64(value);
    } catch {
      throw new Error(
        "Geçersiz attestation ActionHash: " + value
      );
    }
  });
}

function decodeReview(record) {
  const present = record?.entry?.Present;

  if (!present) return null;

  try {
    const bytes = present.entry ?? present;

    if (!(bytes instanceof Uint8Array)) {
      return null;
    }

    return decode(bytes);
  } catch {
    return null;
  }
}

function getReviewAuthor(record) {
  const author = record?.signed_action?.hashed?.content?.author;

  if (!author) return "Bilinmiyor";

  try {
    return encodeHashToBase64(author);
  } catch {
    return "Bilinmiyor";
  }
}

function createElement(tag, className, text) {
  const element = document.createElement(tag);

  if (className) {
    element.className = className;
  }

  if (text !== undefined) {
    element.textContent = text;
  }

  return element;
}

async function loadSummary() {
  const subjectAgent = readAgentKey();

  setStatus("repSummary", "Altı itibar alanı ağdan sorgulanıyor...");

  const summaries = await callReputation(
    "get_contextual_reputation_summary",
    {
      subject_agent: subjectAgent,
      domain: null
    }
  );

  const container = byId("repSummary");
  container.replaceChildren();

  container.appendChild(
    createElement("strong", "", "Alan bazlı itibar özeti")
  );

  for (const summary of summaries) {
    const domainName =
      DOMAINS[summary.domain] || summary.domain;

    const count = Number(summary.review_count ?? 0);
    const average = Number(summary.average_rating ?? 0);

    const item = createElement("div", "rep-domain");

    item.appendChild(
      createElement("strong", "", domainName)
    );

    item.appendChild(
      createElement(
        "div",
        "",
        `Ortalama: ${average.toFixed(2)} / 5`
      )
    );

    item.appendChild(
      createElement(
        "div",
        "rep-muted",
        `Geçerli değerlendirme sayısı: ${count}`
      )
    );

    container.appendChild(item);
  }

  if (!summaries.length) {
    container.appendChild(
      createElement("p", "", "Henüz özet verisi bulunamadı.")
    );
  }
}

async function loadReviews() {
  const subjectAgent = readAgentKey();
  const domain = byId("repDomain").value;

  const list = byId("repReviews");
  list.textContent = "Değerlendirmeler ağdan yükleniyor...";

  const records = await callReputation(
    "get_contextual_reviews",
    {
      subject_agent: subjectAgent,
      domain
    }
  );

  list.replaceChildren();

  list.appendChild(
    createElement(
      "strong",
      "",
      `${DOMAINS[domain]} değerlendirmeleri`
    )
  );

  let visibleCount = 0;

  for (const record of records) {
    const review = decodeReview(record);

    if (!review) continue;

    visibleCount++;

    const item = createElement("div", "rep-review");

    item.appendChild(
      createElement(
        "strong",
        "",
        `★ ${review.rating} / 5`
      )
    );

    if (review.comment) {
      item.appendChild(
        createElement("p", "", review.comment)
      );
    }

    item.appendChild(
      createElement(
        "div",
        "rep-muted",
        "Değerlendiren ajan:"
      )
    );

    item.appendChild(
      createElement(
        "div",
        "rep-hash",
        getReviewAuthor(record)
      )
    );

    const evidence = review.evidence_attestations || [];

    item.appendChild(
      createElement(
        "div",
        "rep-muted",
        `Kanıt kayıtları: ${evidence.length}`
      )
    );

    for (const hash of evidence) {
      try {
        item.appendChild(
          createElement(
            "div",
            "rep-hash",
            encodeHashToBase64(hash)
          )
        );
      } catch {
        item.appendChild(
          createElement(
            "div",
            "rep-hash",
            "Kanıt hash bilgisi okunamadı."
          )
        );
      }
    }

    list.appendChild(item);
  }

  if (visibleCount === 0) {
    list.appendChild(
      createElement(
        "p",
        "",
        "Bu alanda henüz değerlendirme bulunmuyor."
      )
    );
  }
}

byId("repLookup").addEventListener("click", async () => {
  const button = byId("repLookup");

  button.disabled = true;

  try {
    await loadSummary();
  } catch (error) {
    setStatus("repSummary", friendlyError(error));
  } finally {
    button.disabled = false;
  }
});

byId("repShowReviews").addEventListener("click", async () => {
  const button = byId("repShowReviews");

  button.disabled = true;

  try {
    await loadReviews();
  } catch (error) {
    setStatus("repReviews", friendlyError(error));
  } finally {
    button.disabled = false;
  }
});

byId("repDomain").addEventListener("change", async () => {
  try {
    await loadReviews();
  } catch (error) {
    setStatus("repReviews", friendlyError(error));
  }
});

byId("repSubmit").addEventListener("click", async () => {
  const button = byId("repSubmit");

  button.disabled = true;

  setStatus(
    "repStatus",
    "Değerlendirme ve kanıtlar doğrulanıyor..."
  );

  try {
    const subjectAgent = readAgentKey();
    const evidenceAttestations = readEvidenceHashes();

    const domain = byId("repDomain").value;
    const rating = Number(byId("repRating").value);
    const comment =
      byId("repComment").value.trim() || null;

    const reviewHash = await callReputation(
      "create_contextual_review",
      {
        subject_agent: subjectAgent,
        domain,
        rating,
        comment,
        evidence_attestations: evidenceAttestations
      }
    );

    setStatus(
      "repStatus",
      "Değerlendirme Holochain ağına gönderildi.\nActionHash: " +
      encodeHashToBase64(reviewHash)
    );

    byId("repComment").value = "";
    byId("repEvidence").value = "";

    await loadSummary();
    await loadReviews();
  } catch (error) {
    setStatus("repStatus", friendlyError(error));
  } finally {
    button.disabled = false;
  }
});

// END OF FILE
