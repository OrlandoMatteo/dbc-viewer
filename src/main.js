const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;
let signal_card;


function triggerFileUpload() {
  document.getElementById('file-input').click();
}


async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

  if (greetInputEl.value.length > 2)
    greetMsgEl.innerHTML = await invoke("search", { query: greetInputEl.value });
}

async function show_signal(name) {
  signal_card.innerHTML = await invoke("show_signal", { query: name });

}

async function load_dbc(file) {
  let pippo = await invoke("upload_dbc", { base64Data: file });
  console.log(pippo);
}
window.onload = () => {
  document.getElementById('file-input').addEventListener('change', (event) => {
    const file = event.target.files[0];
    const reader = new FileReader();

    reader.onload = (e) => {
      const base64Content = e.target.result.split(',')[1]; // Extract Base64 data
      // Send the Base64 data to your Tauri backend using a Tauri command
      load_dbc(base64Content);
    };

    reader.readAsDataURL(file);
  });
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#signal-input");
  greetMsgEl = document.querySelector("#results");
  document.querySelector("#signal-input").addEventListener("keyup", (e) => {
    e.preventDefault();
    greet();
  });
});

function get_signal(name) {
  signal_card = document.querySelector("#signal_card");
  show_signal(name);
}
