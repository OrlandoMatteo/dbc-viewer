const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// Register listener IMMEDIATELY
(async () => {
  console.log('🔧 Registering file-open listener...');

  await listen('file-open', async (event) => {
    console.log('🎯 FILE OPEN EVENT RECEIVED:', event.payload);

    const { path, filename } = event.payload;

    try {
      console.log('📂 Loading file:', filename);

      // Call the Rust function to load the file
      const result = await invoke('load_file_from_path', { path: path });
      const response = JSON.parse(result);

      console.log('📊 Load result:', response);

      // Update the UI
      if (response.code === 200) {
        document.getElementById('filename').innerHTML = `✅ ${response.message}`;
        document.getElementById('filename').classList.remove('alert-light', 'alert-danger');
        document.getElementById('filename').classList.add('alert-success');

        // Refresh current page if on signals or messages view
        const path = window.location.pathname;
        const page = path.split("/").pop();
        if (page === "signals.html") {
          await get_all_signals();
        } else if (page === "message.html") {
          await get_all_messages();
        }
      } else {
        document.getElementById('filename').innerHTML = `❌ ${response.message}`;
        document.getElementById('filename').classList.remove('alert-light', 'alert-success');
        document.getElementById('filename').classList.add('alert-danger');
      }

    } catch (error) {
      console.error('❌ Error loading file:', error);
      alert('Failed to load file: ' + error);
    }
  });

  console.log('✅ File open listener registered');
})();

let greetInputEl;
let greetMsgEl;
let signal_card;

async function prevHist(){
  console.log("Sending Prev command...");
  const html = await invoke("handle_history", { query: "Prev" });
  console.log("Received response:", html.substring(0, 100)); // show start only
  document.getElementById("signal_card").innerHTML = html;

}
async function nextHist (){
  signal_card = document.querySelector("#signal_card");
  signal_card.innerHTML = await invoke("handle_history", {query:"Next"});
}

function triggerFileUpload() {
  document.getElementById('file-input').click();
}


async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

  if (greetInputEl.value.length > 2)
    greetMsgEl.innerHTML = await invoke("search", { query: greetInputEl.value });
  items = [...document.querySelectorAll(".list-group-item")];
  document.getElementById('pageNumber').innerHTML = currentPage.toString()+" of " + Math.round(1+items.length/pageSize).toString();
  render()
  let results = document.querySelectorAll('.list-group-item')

  results.forEach(function(elem) {
    elem.addEventListener('click', function() {
      var items = document.querySelectorAll('.list-group-item');
      //    // Loop through each element and remove the 'active' class
      items.forEach(function(item) {
        {
          item.classList.remove('active');
        }
      });
      let newTab = event.target
      newTab.classList.add('active')
    })
  })
}

async function show_signal(name) {
  signal_card.innerHTML = await invoke("show_signal", { query: name });
  signal_card = document.querySelector("#signal_card");
}

async function show_message(name) {
  signal_card.innerHTML = await invoke("show_message", { query: name });
  signal_card = document.querySelector("#signal_card");
}

async function is_dbc_loaded() {
  let _response = await invoke("is_dbc_loaded");
  let response = JSON.parse(_response)
  if (response.code == 200) {
    document.getElementById('filename').innerHTML = response.message;
    document.getElementById('filename').classList.remove("alert-light");
    document.getElementById('filename').classList.add("alert-success");
  }
  else {
    document.getElementById('filename').innerHTML = response.message;
  }


}

async function load_dbc(file, filename) {
  let _response = await invoke("upload_dbc", { base64Data: file, filename: filename });
  let response = JSON.parse(_response)
  if (response.code == 200) {
    document.getElementById('filename').innerHTML = response.message;
    document.getElementById('filename').classList.remove("alert-light");
    document.getElementById('filename').classList.add("alert-success");
  }
  else {
    document.getElementById('filename').innerHTML = response.message;
    document.getElementById('filename').classList.remove("alert-light");
    document.getElementById('filename').classList.add("alert-danger");
  }
  var path = window.location.pathname;
  var page = path.split("/").pop();
  if (page == "signals.html") {
    get_all_signals()
  }
  else if (page == "message.html") {
    get_all_messages()
  }
}
async function get_all_signals() {
  show_spinner()
  document.getElementById("page").innerHTML = await invoke("get_all_signals");
}
async function get_all_messages() {
  show_spinner()
  document.getElementById("page").innerHTML = await invoke("get_all_messages");
}

window.onload = () => {
  let file_input = document.getElementById('file-input')
  if (file_input) {
    file_input.addEventListener('change', (event) => {
      const file = event.target.files[0];
      const reader = new FileReader();

      reader.onload = (e) => {
        const base64Content = e.target.result.split(',')[1]; // Extract Base64 data
        // Send the Base64 data to your Tauri backend using a Tauri command
        load_dbc(base64Content, file.name);
      };

      reader.readAsDataURL(file);

    });
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#signal-input");
  greetMsgEl = document.querySelector("#results");
  document.getElementById("nextBtn").onclick = () => {
    if (currentPage * pageSize < items.length) {
      currentPage++;
      render();
    }
    document.getElementById('pageNumber').innerHTML = currentPage.toString()+" of " + Math.round(1+items.length/pageSize).toString();
  };

  document.getElementById("prevBtn").onclick = () => {
    if (currentPage > 1) {
      currentPage--;
      render();
    }
    document.getElementById('pageNumber').innerHTML = currentPage.toString()+" of " + Math.round(1+items.length/pageSize).toString();
  };
  let signal_input = document.querySelector("#signal-input")
  if (signal_input) {
    signal_input.addEventListener("keyup", (e) => {
      e.preventDefault();
      greet();
    });
    is_dbc_loaded();
  }
  // document.getElementById("prevHist").onclick = () => {
  //   prevHist();
  // };
  //
  // document.getElementById("nextHist").onclick = () => {
  //   nextHist();
  // };

});




function get_signal(name) {

  signal_card = document.querySelector("#signal_card");
  show_signal(name);
}

function get_message(name) {
  signal_card = document.querySelector("#signal_card");
  show_message(name);
}

// Get the car image element
const carImage = document.querySelector('#car-container img');

// Define a function to start the animation
function startAnimation() {
  carImage.classList.add('animate'); // Add a class to trigger the animation
}

// Define a function to stop the animation
function stopAnimation() {
  carImage.classList.remove('animate'); // Remove the class to stop the animation
}

function show_spinner() {
  document.getElementById("page").innerHTML = `<div class="spinner-border" role="status">
  <span class="visually-hidden">Loading...</span>
</div>`
}

const pageSize = 10;
let currentPage = 1;

let items;

function render() {
  items.forEach((li, i) => {
    li.style.display = (i >= (currentPage-1)*pageSize && i < currentPage*pageSize)
        ? "flex" : "none";
  });
}



