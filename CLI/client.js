#!/usr/bin/env bun
import { intro, text, isCancel, outro, spinner } from "@clack/prompts";
import { dim, red, cyan, green } from "kolorist";

const cy = "17B890";

const hexToRgb = (hex) => {
  const bigint = parseInt(hex, 16);
  const r = (bigint >> 16) & 255;
  const g = (bigint >> 8) & 255;
  const b = bigint & 255;
  return { r, g, b };
};

const echo = (text, color = cy) => {
  // Convert hex color to rgb
  const rgb = hexToRgb(color);
  // Apply rgb color and text to output
  process.stdout.write(
    `\x1b[38;2;${rgb.r};${rgb.g};${rgb.b}\m${text}\x1b[0m\n`
  );
};

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const stream = async (text) => {
  const rgb = hexToRgb("17B890");
  await text.split(" ").forEach(async (x) => {
    process.stdout.write(`\x1b[38;2;${rgb.r};${rgb.g};${rgb.b}\m${x} \x1b[0m`);
    await sleep(100);
  });
};

async function saturn(query) {
  const saturnResponse = await fetch("http://localhost:2223/query", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ query }),
  });
  const data = await saturnResponse.json();
  const response = data?.response ? data?.response : data?.error;
  return response;
}

async function conversation() {
  echo("  └────────────────────╼", cy);
  intro("Starting new conversation");
  const msgYou = `${"You"}:`;
  const userPrompt = await text({
    message: `${cyan(msgYou)}`,
    placeholder: `send a message ('exit' to quit)`,
    validate: (value) => {
      if (!value) return "Please enter a prompt.";
    },
  });
  if (isCancel(userPrompt) || userPrompt === "exit") {
    outro("Goodbye!");
    process.exit(0);
  }
  const infoSpin = spinner();
  infoSpin.start(`THINKING...`);

  const saturnResponse = await saturn(userPrompt);

  infoSpin.stop(`${green("Web Search:")}`);
  await stream(`│\n${saturnResponse}`);
  console.log("");
  console.log("");
}

await conversation();
