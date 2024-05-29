const { readFile } = window.__TAURI__.fs;
const invoke = window.__TAURI__.core.invoke;
// import { invoke } from "@tauri-apps/api/core";

let greetInputEl;
let greetMsgEl;

async function tauri1(v) {
  greetMsgEl.textContent = invoke("tauri_cmd_1", { input: v });
  greetMsgEl.textContent = await readFile("tauri_cmd_1", {
    dir: "toto",
  });
}

async function greet() {
  await tauri1(greetInputEl.value);
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document
    .querySelector("#greet-button")
    .addEventListener("click", () => greet());
});

// invoke("tauri_cmd_2", { input: 3 });
