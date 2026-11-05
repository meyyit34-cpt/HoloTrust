import { decode, encode } from "@msgpack/msgpack";
import { encodeHashToBase64 } from "@holochain/client";

const BASE = "http://127.0.0.1:" + (process.env.WEB_PORT ?? "8080");

function check(condition, message) {
  if (!condition) {
    throw new Error("PROXY FAIL: " + message);
  }
  console.log("PROXY PASS: " + message);
}

async function call(fnName, payload) {
  const response = await fetch(BASE + "/hc/call/" + fnName, {
    method: "POST",
    headers: { "Content-Type": "application/msgpack" },
    body: encode(payload)
  });
  const body = decode(new Uint8Array(await response.arrayBuffer()));
  return { status: response.status, body };
}

async function main() {
  const info = await fetch(BASE + "/hc/info");
  check(info.ok, "GET /hc/info responds");
  const infoBody = decode(new Uint8Array(await info.arrayBuffer()));
  const agentKey = infoBody.agent_pub_key;
  check(agentKey instanceof Uint8Array, "agent public key arrives as Uint8Array");
  check(
    encodeHashToBase64(agentKey).startsWith("uhCAk"),
    "agent public key encodes with uhCAk prefix"
  );

  const page = await fetch(BASE + "/");
  check(page.ok, "static index.html is served");

  const mine = await call("get_my_profile", null);
  check(mine.status === 200, "get_my_profile via proxy returns 200");
  check(mine.body.result !== null, "existing profile found via proxy");

  const hash = mine.body.result?.signed_action?.hashed?.hash;
  check(hash instanceof Uint8Array, "record action hash survives the proxy");

  const updated = await call("update_profile", {
    action_hash: hash,
    display_name: "Proxy Updated",
    agent_type: "Service",
    description: null
  });
  check(updated.status === 200, "update_profile via proxy returns 200");
  check(
    updated.body.result instanceof Uint8Array,
    "update_profile returns an action hash"
  );

  const reviews = await call("get_reviews", { subject_agent: agentKey });
  check(reviews.status === 200, "get_reviews via proxy returns 200");
  check(
    Array.isArray(reviews.body.result) && reviews.body.result.length === 0,
    "get_reviews returns an empty list for an unreviewed agent"
  );

  const summary = await call("get_reputation_summary", {
    subject_agent: agentKey
  });
  check(summary.status === 200, "get_reputation_summary returns 200");
  check(
    summary.body.result.review_count === 0 &&
      summary.body.result.average_rating === 0,
    "summary is 0 reviews and 0 average"
  );

  const selfReview = await call("create_review", {
    subject_agent: agentKey,
    rating: 5,
    comment: null
  });
  console.log("PROXY INFO: self review error = " + selfReview.body.error);
  check(selfReview.status === 500, "self review is rejected via proxy");
  check(
    String(selfReview.body.error).includes("cannot review"),
    "self review error message reaches the client"
  );

  const forbidden = await call("evil_function", null);
  check(forbidden.status === 403, "non-allowlisted function is rejected");

  console.log("PROXY DONE: all checks passed");
}

main().catch(error => {
  console.error(String(error?.stack ?? error));
  process.exit(1);
});
