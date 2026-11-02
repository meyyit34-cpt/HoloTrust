
import { callIdentity } from "./holochain.js";
import {
  decodeHashFromBase64,
  encodeHashToBase64
} from "https://esm.sh/@holochain/client@0.21.0";

const style = document.createElement("style");
style.textContent = `
#reputationPanel {
  margin: 28px auto;
  max-width: 760px;
  padding: 24px;
  color: #e8edf7;
  background: #111827;
  border: 1px solid #293449;
  border-radius: 16px;
  font-family: inherit;
}
#reputationPanel h2 { margin-top: 0; }
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
}
#reputationPanel button {
  padding: 11px 16px;
  color: #07111f;
  background: #66e3c4;
  border: 0;
  border-radius: 8px;
  font-weight: 700;
  cursor: pointer;
}
#reputationPanel button:disabled { opacity: .5; }
#reputationPanel .rep-result {
  margin-top: 14px;
  padding: 14px;
  background: #0b1220;
  border-radius: 8px;
  overflow-wrap: anywhere;
}
#reputationPanel label {
  display: block;
  margin-top: 12px;
  color: #b9c5d8;
}
#reputationPanel .rep-muted {
  color: #9aa9bd;
  font-size: 13px;
}
`;
document.head.appendChild(style);

const panel = document.createElement("section");
panel.id = "reputationPanel";
panel.innerHTML = `
  <h2>HoloTrust Reputation</h2>
  <p class="rep-muted">
    Ajan kimliğini girerek itibar puanını sorgula veya değerlendirme yayınla.
  </p>

  <label for="repAgentKey">Hedef Agent Public Key</label>
  <input id="repAgentKey" autocomplete="off" placeholder="uhCAk...">

  <button id="repLookup">İtibar puanını sorgula</button>

  <div id="repSummary" class="rep-result" aria-live="polite">
    Henüz sorgulama yapılmadı.
  </div>

  <hr style="border-color:#293449;margin:24px 0">

  <h3>Değerlendirme yayınla</h3>

  <label for="repRating">Puan</label>
  <select id="repRating">
    <option value="5">5 — Çok iyi</option>
    <option value="4">4 — İyi</option>
    <option value="3">3 — Orta</option>
    <option value="2">2 — Zayıf</option>
    <option value="1">1 — Çok kötü</option>
  </select>

  <label for="repComment">Yorum (isteğe bağlı, en fazla 500 karakter)</label>
  <textarea id="repComment" maxlength="500" rows="3"
    placeholder="Deneyimini açıkla..."></textarea>

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

function readAgentKey() {
  const value = byId("repAgentKey").value.trim();

  if (!value) {
    throw new Error("Hedef Agent Public Key gerekli.");
  }

  try {
    return decodeHashFromBase64(value);
  } catch {
    throw new Error("Agent Public Key geçerli Base64 formatında değil.");
  }
}

byId("repLookup").addEventListener("click", async () => {
  const button = byId("repLookup");
  button.disabled = true;
  setStatus("repSummary", "İtibar ağı sorgulanıyor...");

  try {
    const subjectAgent = readAgentKey();
    const summary = await callIdentity(
      "get_reputation_summary",
      { subject_agent: subjectAgent }
    );

    const average = Number(summary.average_rating ?? 0);
    const count = Number(summary.review_count ?? 0);

    setStatus(
      "repSummary",
      `Ortalama puan: ${average.toFixed(2)} / 5 | Değerlendirme sayısı: ${count}`
    );
  } catch (error) {
    setStatus("repSummary", error.message || "Sorgulama başarısız.");
  } finally {
    button.disabled = false;
  }
});

byId("repSubmit").addEventListener("click", async () => {
  const button = byId("repSubmit");
  button.disabled = true;
  setStatus("repStatus", "Değerlendirme ağa gönderiliyor...");

  try {
    const subjectAgent = readAgentKey();
    const rating = Number(byId("repRating").value);
    const comment = byId("repComment").value.trim() || null;

    const reviewHash = await callIdentity("create_review", {
      subject_agent: subjectAgent,
      rating,
      comment
    });

    setStatus(
      "repStatus",
      `Değerlendirme yayınlandı. Kayıt: ${encodeHashToBase64(reviewHash)}`
    );

    byId("repComment").value = "";
    byId("repLookup").click();
  } catch (error) {
    setStatus("repStatus", error.message || "Değerlendirme yayınlanamadı.");
  } finally {
    button.disabled = false;
  }
});

