const { invoke } = window.__TAURI__.tauri;

let greetInputEl;
let greetMsgEl;

async function tauri1(v) {
  greetMsgEl.textContent = await invoke("tauri_cmd_1", { input: v });
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
