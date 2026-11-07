import { decode, encode } from "@msgpack/msgpack";
import { encodeHashToBase64 } from "@holochain/client";

const BASE = "http://127.0.0.1:" + (process.env.WEB_PORT ?? "8080");

function check(condition, message) {
  if (!condition) {
    throw new Error("PROXY FAIL: " + message);
  }

  console.log("PROXY PASS: " + message);
}

async function call(zomeName, fnName, payload) {
  const url =
    BASE +
    "/hc/call/" +
    encodeURIComponent(zomeName) +
    "/" +
    encodeURIComponent(fnName);

  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/msgpack" },
    body: encode(payload)
  });

  const body = decode(
    new Uint8Array(await response.arrayBuffer())
  );

  return {
    status: response.status,
    body
  };
}

async function main() {
  const info = await fetch(BASE + "/hc/info");
  check(info.ok, "GET /hc/info responds");

  const infoBody = decode(
    new Uint8Array(await info.arrayBuffer())
  );

  const agentKey = infoBody.agent_pub_key;

  check(
    agentKey instanceof Uint8Array,
    "agent public key arrives as Uint8Array"
  );

  check(
    encodeHashToBase64(agentKey).startsWith("uhCAk"),
    "agent public key encodes with uhCAk prefix"
  );

  const page = await fetch(BASE + "/");
  check(page.ok, "static index.html is served");

  const pageText = await page.text();

  check(
    pageText.includes("reputation-ui.js"),
    "index.html loads the contextual reputation interface"
  );

  const expectedDomains = [
    "commerce",
    "professional",
    "hospitality",
    "technical",
    "community",
    "reliability"
  ];

  for (const domain of expectedDomains) {
    check(
      pageText.includes(`value="${domain}"`),
      "index.html contains the " + domain + " domain option"
    );
  }

  const reputationScript = await fetch(BASE + "/reputation-ui.js");

  check(
    reputationScript.ok,
    "contextual reputation JavaScript is served"
  );

  const reputationScriptText = await reputationScript.text();

  check(
    reputationScriptText.includes("create_contextual_review") &&
      reputationScriptText.includes("get_contextual_reviews") &&
      reputationScriptText.includes("get_contextual_reviews_with_provenance") &&
      reputationScriptText.includes("get_contextual_reputation_summary"),
    "reputation interface references all four contextual APIs"
  );

  const mine = await call(
    "identity_coordinator",
    "get_my_profile",
    null
  );

  if (mine.status !== 200) {
    throw new Error(
      "PROXY FAIL: get_my_profile returned HTTP " +
      mine.status +
      " - " +
      String(mine.body?.error ?? "unknown error")
    );
  }

  check(
    mine.status === 200,
    "get_my_profile via proxy returns 200"
  );

  check(
    mine.body.result !== null,
    "existing profile found via proxy"
  );

  const hash = mine.body.result?.signed_action?.hashed?.hash;

  check(
    hash instanceof Uint8Array,
    "record action hash survives the proxy"
  );

  const updated = await call(
    "identity_coordinator",
    "update_profile",
    {
      action_hash: hash,
      display_name: "Proxy Updated",
      agent_type: "Service",
      description: null
    }
  );

  check(
    updated.status === 200,
    "update_profile via proxy returns 200"
  );

  check(
    updated.body.result instanceof Uint8Array,
    "update_profile returns an action hash"
  );

  const reviews = await call(
    "identity_coordinator",
    "get_reviews",
    {
      subject_agent: agentKey
    }
  );

  check(
    reviews.status === 200,
    "legacy get_reviews via proxy returns 200"
  );

  check(
    Array.isArray(reviews.body.result) &&
      reviews.body.result.length === 0,
    "legacy get_reviews returns an empty list for an unreviewed agent"
  );

  const legacySummary = await call(
    "identity_coordinator",
    "get_reputation_summary",
    {
      subject_agent: agentKey
    }
  );

  check(
    legacySummary.status === 200,
    "legacy get_reputation_summary returns 200"
  );

  check(
    legacySummary.body.result.review_count === 0 &&
      legacySummary.body.result.average_rating === 0,
    "legacy summary is 0 reviews and 0 average"
  );

  const contextualReviews = await call(
    "reputation_coordinator",
    "get_contextual_reviews",
    {
      subject_agent: agentKey,
      domain: null
    }
  );

  check(
    contextualReviews.status === 200,
    "contextual get_contextual_reviews via proxy returns 200"
  );

  check(
    Array.isArray(contextualReviews.body.result) &&
      contextualReviews.body.result.length === 0,
    "contextual reviews returns an empty list for a new agent"
  );

  const contextualSummary = await call(
    "reputation_coordinator",
    "get_contextual_reputation_summary",
    {
      subject_agent: agentKey,
      domain: null
    }
  );

  check(
    contextualSummary.status === 200,
    "contextual reputation summary via proxy returns 200"
  );

  const contextualProvenance = await call(
    "reputation_coordinator",
    "get_contextual_reviews_with_provenance",
    {
      subject_agent: agentKey,
      domain: null
    }
  );

  check(
    contextualProvenance.status === 200,
    "contextual review provenance API via proxy returns 200"
  );

  check(
    Array.isArray(contextualProvenance.body.result) &&
      contextualProvenance.body.result.length === 0,
    "contextual review provenance returns an empty list for a new agent"
  );

  const summaries = contextualSummary.body.result;

  check(
    Array.isArray(summaries) && summaries.length === 6,
    "contextual summary returns all six reputation domains"
  );

  for (const domain of expectedDomains) {
    const summary = summaries.find(item => item.domain === domain);

    check(
      Boolean(summary),
      "contextual summary includes " + domain
    );

    check(
      summary.review_count === 0 &&
        summary.average_rating === 0,
      domain + " starts with zero reviews and zero average"
    );
  }

  const selfReview = await call(
    "reputation_coordinator",
    "create_contextual_review",
    {
      subject_agent: agentKey,
      domain: "commerce",
      rating: 5,
      comment: null,
      evidence_attestations: []
    }
  );

  check(
    selfReview.status === 500,
    "contextual self-review is rejected via proxy"
  );

  check(
    String(selfReview.body.error).includes("cannot review"),
    "contextual self-review rejection message reaches the client"
  );

  const legacySelfReview = await call(
    "identity_coordinator",
    "create_review",
    {
      subject_agent: agentKey,
      rating: 5,
      comment: null
    }
  );

  check(
    legacySelfReview.status === 500,
    "legacy self-review is still rejected via proxy"
  );

  check(
    String(legacySelfReview.body.error).includes("cannot review"),
    "legacy self-review rejection message reaches the client"
  );

  const forbidden = await call(
    "identity_coordinator",
    "evil_function",
    null
  );

  check(
    forbidden.status === 403,
    "non-allowlisted function is rejected"
  );

  console.log("PROXY DONE: all checks passed");
}

main().catch(error => {
  console.error(String(error?.stack ?? error));
  process.exit(1);
});
