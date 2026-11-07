function evidenceStatusLabel(status) {
  const labels = {
    Active: "Aktif",
    Revoked: "İptal edilmiş",
    Expired: "Süresi dolmuş",
    Superseded: "Yeni kayıtla değiştirilmiş",
    Missing: "Kayıt bulunamadı",
    Invalid: "Geçersiz kayıt",
    Unavailable: "Durum şu anda doğrulanamıyor"
  };

  return labels[status] || "Bilinmeyen durum";
}

async function loadReviews() {
  const subjectAgent = readAgentKey();
  const domain = byId("repDomain").value;

  const list = byId("repReviews");
  list.textContent = "Değerlendirmeler ve kanıt geçmişi ağdan yükleniyor...";

  const records = await callReputation(
    "get_contextual_reviews_with_provenance",
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
      `${DOMAINS[domain]} değerlendirmeleri ve kanıt geçmişi`
    )
  );

  let visibleCount = 0;

  for (const reviewRecord of records) {
    const review = reviewRecord?.review;
    if (!review) continue;

    visibleCount++;

    const item = createElement("div", "rep-review");

    item.appendChild(
      createElement("strong", "", `★ ${review.rating} / 5`)
    );

    if (review.comment) {
      item.appendChild(createElement("p", "", review.comment));
    }

    item.appendChild(
      createElement("div", "rep-muted", "Değerlendiren ajan:")
    );

    try {
      item.appendChild(
        createElement(
          "div",
          "rep-hash",
          encodeHashToBase64(reviewRecord.author)
        )
      );
    } catch {
      item.appendChild(
        createElement("div", "rep-hash", "Ajan bilgisi okunamadı.")
      );
    }

    try {
      item.appendChild(
        createElement(
          "div",
          "rep-hash",
          "Değerlendirme ActionHash: " +
            encodeHashToBase64(reviewRecord.action_hash)
        )
      );
    } catch {
      item.appendChild(
        createElement("div", "rep-hash", "ActionHash okunamadı.")
      );
    }

    const evidence = Array.isArray(reviewRecord.evidence)
      ? reviewRecord.evidence
      : [];

    item.appendChild(
      createElement("div", "rep-muted", `Kanıt kayıtları: ${evidence.length}`)
    );

    for (const proof of evidence) {
      const evidenceItem = createElement("div", "rep-domain");

      try {
        evidenceItem.appendChild(
          createElement(
            "div",
            "rep-hash",
            "Attestation ActionHash: " + encodeHashToBase64(proof.action_hash)
          )
        );
      } catch {
        evidenceItem.appendChild(
          createElement("div", "rep-hash", "Attestation ActionHash okunamadı.")
        );
      }

      evidenceItem.appendChild(
        createElement(
          "strong",
          "",
          "Durum: " + evidenceStatusLabel(proof.status)
        )
      );

      if (proof.issuer) {
        try {
          evidenceItem.appendChild(
            createElement(
              "div",
              "rep-hash",
              "Düzenleyen: " + encodeHashToBase64(proof.issuer)
            )
          );
        } catch {
          evidenceItem.appendChild(
            createElement("div", "rep-hash", "Düzenleyen bilgisi okunamadı.")
          );
        }
      }

      if (proof.issued_at !== null && proof.issued_at !== undefined) {
        evidenceItem.appendChild(
          createElement(
            "div",
            "rep-muted",
            "Oluşturulma zamanı: " + String(proof.issued_at)
          )
        );
      }

      if (proof.attestation) {
        evidenceItem.appendChild(
          createElement(
            "div",
            "",
            "İddia türü: " + String(proof.attestation.claim_type ?? "Bilinmiyor")
          )
        );

        evidenceItem.appendChild(
          createElement(
            "p",
            "",
            "İddia: " + String(proof.attestation.claim ?? "Açıklama yok")
          )
        );

        try {
          evidenceItem.appendChild(
            createElement(
              "div",
              "rep-hash",
              "İlgili ajan: " + encodeHashToBase64(proof.attestation.subject)
            )
          );
        } catch {
          evidenceItem.appendChild(
            createElement("div", "rep-hash", "İlgili ajan bilgisi okunamadı.")
          );
        }
      } else {
        evidenceItem.appendChild(
          createElement(
            "p",
            "rep-muted",
            "Bu kanıt için doğrulanabilir attestation içeriği alınamadı."
          )
        );
      }

      item.appendChild(evidenceItem);
    }

    list.appendChild(item);
  }

  if (visibleCount === 0) {
    list.appendChild(
      createElement("p", "", "Bu alanda henüz değerlendirme bulunmuyor.")
    );
  }
}
