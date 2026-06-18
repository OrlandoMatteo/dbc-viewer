const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const pageSize = 10;
let currentPage = 1;
let searchResults = [];
let greetInputEl;
let greetMsgEl;

(async () => {
  await listen("file-open", async (event) => {
    const { path: filePath } = event.payload;

    try {
      const response = await invoke("load_file_from_path", { path: filePath });
      setFileStatus(response);
      await refreshCurrentPage();
    } catch (error) {
      alert(`Failed to load file: ${error}`);
    }
  });
})();

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function escapeAttr(value) {
  return escapeHtml(value);
}

function valueOf(object, ...keys) {
  for (const key of keys) {
    if (object?.[key] !== undefined && object?.[key] !== null) {
      return object[key];
    }
  }
  return "";
}

function formatHex(value) {
  if (value === "" || value === undefined || value === null) {
    return "";
  }
  const number = Number(value);
  return Number.isFinite(number) ? `0x${number.toString(16).toUpperCase()}` : value;
}

function safeId(prefix, raw, index) {
  const safe = String(raw || "item").replace(/[^a-zA-Z0-9_-]/g, "-");
  return `${prefix}-${index}-${safe}`;
}

function setFileStatus(response) {
  const filenameEl = document.getElementById("filename");
  if (!filenameEl) {
    return;
  }

  filenameEl.textContent = response.message || "No DBC loaded";
  filenameEl.classList.remove("alert-light", "alert-success", "alert-danger", "alert-warning");
  filenameEl.classList.add(response.loaded ? "alert-success" : "alert-light");

  if (response.loaded && response.warnings?.length) {
    filenameEl.classList.remove("alert-success");
    filenameEl.classList.add("alert-warning");
    filenameEl.title = response.warnings.join("\n");
  }
}

async function refreshCurrentPage() {
  const page = window.location.pathname.split("/").pop();
  if (page === "signals.html") {
    await get_all_signals();
  } else if (page === "messages.html") {
    await get_all_messages();
  } else if (greetInputEl?.value?.length > 2) {
    await greet();
  }
}

function triggerFileUpload() {
  document.getElementById("file-input")?.click();
}

async function prevHist() {
  try {
    const view = await invoke("handle_history", { query: "Prev" });
    renderViewItem(view);
  } catch (error) {
    console.debug(error);
  }
}

async function nextHist() {
  try {
    const view = await invoke("handle_history", { query: "Next" });
    renderViewItem(view);
  } catch (error) {
    console.debug(error);
  }
}

async function greet() {
  if (!greetInputEl || !greetMsgEl) {
    return;
  }

  const query = greetInputEl.value.trim();
  searchResults = query.length > 2 ? await invoke("search", { query }) : [];
  currentPage = 1;
  renderSearchResults();
}

function renderSearchResults() {
  if (!greetMsgEl) {
    return;
  }

  const start = (currentPage - 1) * pageSize;
  const visibleResults = searchResults.slice(start, start + pageSize);

  if (visibleResults.length === 0) {
    greetMsgEl.innerHTML = '<div class="text-muted">No results</div>';
  } else {
    greetMsgEl.innerHTML = `
      <div class="list-group">
        ${visibleResults.map(renderSearchResult).join("")}
      </div>
    `;
  }

  const prevBtn = document.getElementById("prevBtn");
  const nextBtn = document.getElementById("nextBtn");
  if (prevBtn) {
    prevBtn.disabled = currentPage <= 1;
  }
  if (nextBtn) {
    nextBtn.disabled = currentPage * pageSize >= searchResults.length;
  }
}

function renderSearchResult(result) {
  const action = result.kind === "message" ? "view-message" : "view-signal";
  const badgeClass = result.kind === "message" ? "bg-message text-dark" : "bg-signal";
  const badgeText = result.kind === "message" ? "Message" : "Signal";

  return `
    <button type="button"
      class="list-group-item list-group-item-action d-flex justify-content-between align-items-center gap-3"
      data-action="${action}"
      data-name="${escapeAttr(result.name)}">
      <span>
        <span class="fw-semibold">${escapeHtml(result.name)}</span>
        <span class="text-muted ms-2">${escapeHtml(result.id)}</span>
      </span>
      <span class="badge ${badgeClass}">${badgeText}</span>
    </button>
  `;
}

async function show_signal(name) {
  const view = await invoke("show_signal", { query: name });
  renderViewItem(view);
}

async function show_message(name) {
  const view = await invoke("show_message", { query: name });
  renderViewItem(view);
}

async function is_dbc_loaded() {
  const response = await invoke("is_dbc_loaded");
  setFileStatus(response);
}

async function load_dbc(file, filename) {
  const response = await invoke("upload_dbc", { base64Data: file, filename });
  setFileStatus(response);
  await refreshCurrentPage();
}

async function get_all_signals() {
  show_spinner();
  const page = document.getElementById("page");
  if (!page) {
    return;
  }

  const signals = await invoke("get_all_signals");
  if (!signals.length) {
    page.innerHTML = '<div class="text-muted">No DBC loaded</div>';
    return;
  }

  page.innerHTML = `
    <div class="accordion" id="signalsAccordion">
      ${signals.map((signal, index) => renderSignalAccordionItem(signal, "signalsAccordion", index)).join("")}
    </div>
  `;
}

async function get_all_messages() {
  show_spinner();
  const page = document.getElementById("page");
  if (!page) {
    return;
  }

  const details = await invoke("get_all_messages");
  if (!details.length) {
    page.innerHTML = '<div class="text-muted">No DBC loaded</div>';
    return;
  }

  page.innerHTML = `
    <div class="accordion" id="messagesAccordion">
      ${details.map((detail, index) => renderMessageAccordionItem(detail, "messagesAccordion", index)).join("")}
    </div>
  `;
}

function renderViewItem(view) {
  const signalCard = document.getElementById("signal_card");
  if (!signalCard || !view) {
    return;
  }

  if (view.kind === "signal") {
    signalCard.innerHTML = renderSignalCard(view.item);
  } else if (view.kind === "message") {
    signalCard.innerHTML = renderMessageCard(view.item);
  }
}

function renderSignalCard(signal) {
  return `
    <div class="card">
      <div class="card-body">
        <h5 class="card-title">${escapeHtml(valueOf(signal, "name"))}</h5>
        <h6 class="card-subtitle mb-2 text-muted">${escapeHtml(valueOf(signal, "label"))}</h6>
        ${renderSignalDetails(signal)}
      </div>
    </div>
  `;
}

function renderSignalDetails(signal) {
  const msgName = valueOf(signal, "msgName", "msg_name");
  const fields = [
    ["Start bit", valueOf(signal, "startBit", "start_bit")],
    ["Bit length", valueOf(signal, "bitLength", "bit_length")],
    ["Little endian", valueOf(signal, "isLittleEndian", "is_little_endian") ? "Yes" : "No"],
    ["Signed", valueOf(signal, "isSigned", "is_signed") ? "Yes" : "No"],
    ["Factor", valueOf(signal, "factor")],
    ["Offset", valueOf(signal, "offset")],
    ["Min", valueOf(signal, "min")],
    ["Max", valueOf(signal, "max")],
    ["Source unit", valueOf(signal, "sourceUnit", "source_unit")],
    ["Signal ID", valueOf(signal, "sig_id")],
    ["Interval", valueOf(signal, "interval")],
    ["Category", valueOf(signal, "category")],
    ["Msg ID", formatHex(valueOf(signal, "msgId", "msg_id"))],
  ];

  return `
    <div class="row g-3 mt-2">
      ${fields.map(renderField).join("")}
      <div class="col-md-6">
        <div class="d-flex justify-content-between border-bottom pb-2">
          <span class="fw-semibold">Msg Name:</span>
          <span class="text-end">
            ${msgName
              ? `<button type="button" class="btn btn-link p-0 align-baseline" data-action="view-message" data-name="${escapeAttr(msgName)}">${escapeHtml(msgName)}</button>`
              : ""}
          </span>
        </div>
      </div>
    </div>
    ${renderStatesTable(valueOf(signal, "states") || [])}
  `;
}

function renderSignalAccordionItem(signal, parentId, index) {
  const id = safeId("signal", valueOf(signal, "name"), index);

  return `
    <div class="accordion-item border-bottom-0">
      <h2 class="accordion-header" id="${id}-header">
        <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#${id}" aria-expanded="false" aria-controls="${id}">
          ${escapeHtml(valueOf(signal, "name"))}
        </button>
      </h2>
      <div id="${id}" class="accordion-collapse collapse" data-bs-parent="#${parentId}">
        <div class="accordion-body border-signal">
          ${renderSignalDetails(signal)}
        </div>
      </div>
    </div>
  `;
}

function renderMessageCard(detail) {
  return `
    <div class="card">
      <div class="card-body">
        ${renderMessageDetails(detail)}
      </div>
    </div>
  `;
}

function renderMessageDetails(detail) {
  const message = detail.message;
  const signals = detail.signals || [];

  return `
    <h5 class="card-title">${escapeHtml(message.name)}</h5>
    <div class="row g-3 mt-2">
      ${[
        ["CAN ID", formatHex(message.can_id)],
        ["PGN", message.pgn],
        ["Source", message.source],
        ["Priority", message.priority],
        ["DLC", message.dlc],
        ["Extended frame", message.isExtendedFrame ? "Yes" : "No"],
      ].map(renderField).join("")}
    </div>
    <h6 class="fw-bold mt-4">Signals</h6>
    ${signals.length
      ? `<div class="list-group">${signals.map((signal) => `
          <button type="button" class="list-group-item list-group-item-action" data-action="view-signal" data-name="${escapeAttr(valueOf(signal, "name"))}">
            ${escapeHtml(valueOf(signal, "name"))}
          </button>
        `).join("")}</div>`
      : '<div class="text-muted">No signal details found</div>'}
  `;
}

function renderMessageAccordionItem(detail, parentId, index) {
  const message = detail.message;
  const id = safeId("message", message.name, index);
  const signalsParentId = `${id}-signals`;

  return `
    <div class="accordion-item border-bottom-0">
      <h2 class="accordion-header" id="${id}-header">
        <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#${id}" aria-expanded="false" aria-controls="${id}">
          ${escapeHtml(message.name)}
        </button>
      </h2>
      <div id="${id}" class="accordion-collapse collapse" data-bs-parent="#${parentId}">
        <div class="accordion-body border border-message">
          <div class="row g-3 mb-3">
            ${[
              ["CAN ID", formatHex(message.can_id)],
              ["PGN", message.pgn],
              ["Source", message.source],
              ["Priority", message.priority],
              ["DLC", message.dlc],
              ["Extended frame", message.isExtendedFrame ? "Yes" : "No"],
            ].map(renderField).join("")}
          </div>
          <div class="accordion" id="${signalsParentId}">
            ${(detail.signals || []).map((signal, signalIndex) => renderSignalAccordionItem(signal, signalsParentId, signalIndex)).join("")}
          </div>
        </div>
      </div>
    </div>
  `;
}

function renderField([label, value]) {
  return `
    <div class="col-md-6">
      <div class="d-flex justify-content-between border-bottom pb-2 gap-3">
        <span class="fw-semibold">${escapeHtml(label)}:</span>
        <span class="text-end">${escapeHtml(value)}</span>
      </div>
    </div>
  `;
}

function renderStatesTable(states) {
  if (!states.length) {
    return "";
  }

  return `
    <div class="mt-4">
      <h6 class="fw-bold">States</h6>
      <table class="table table-hover">
        <thead><tr><th>Value</th><th>State</th></tr></thead>
        <tbody>
          ${states.map((state) => `<tr><td>${escapeHtml(state.value)}</td><td>${escapeHtml(state.state)}</td></tr>`).join("")}
        </tbody>
      </table>
    </div>
  `;
}

function get_signal(name) {
  show_signal(name);
}

function get_message(name) {
  show_message(name);
}

function show_spinner() {
  const page = document.getElementById("page");
  if (!page) {
    return;
  }

  page.innerHTML = `<div class="spinner-border" role="status">
    <span class="visually-hidden">Loading...</span>
  </div>`;
}

function setupFileInput() {
  const fileInput = document.getElementById("file-input");
  if (!fileInput) {
    return;
  }

  fileInput.addEventListener("change", (event) => {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }

    const reader = new FileReader();
    reader.onload = (readerEvent) => {
      const base64Content = readerEvent.target.result.split(",")[1];
      load_dbc(base64Content, file.name);
    };
    reader.readAsDataURL(file);
  });
}

function setupSearchPage() {
  greetInputEl = document.querySelector("#signal-input");
  greetMsgEl = document.querySelector("#results");

  const nextBtn = document.getElementById("nextBtn");
  const prevBtn = document.getElementById("prevBtn");

  if (nextBtn) {
    nextBtn.onclick = () => {
      if (currentPage * pageSize < searchResults.length) {
        currentPage++;
        renderSearchResults();
      }
    };
  }

  if (prevBtn) {
    prevBtn.onclick = () => {
      if (currentPage > 1) {
        currentPage--;
        renderSearchResults();
      }
    };
  }

  if (greetInputEl) {
    greetInputEl.addEventListener("keyup", (event) => {
      event.preventDefault();
      greet();
    });
  }

  renderSearchResults();
}

document.addEventListener("click", (event) => {
  const target = event.target.closest("[data-action]");
  if (!target) {
    return;
  }

  const name = target.dataset.name;
  if (target.dataset.action === "view-signal") {
    get_signal(name);
  } else if (target.dataset.action === "view-message") {
    get_message(name);
  }
});

window.addEventListener("DOMContentLoaded", () => {
  setupFileInput();
  setupSearchPage();
  is_dbc_loaded();
});

window.prevHist = prevHist;
window.nextHist = nextHist;
window.triggerFileUpload = triggerFileUpload;
window.greet = greet;
window.show_signal = show_signal;
window.show_message = show_message;
window.is_dbc_loaded = is_dbc_loaded;
window.load_dbc = load_dbc;
window.get_all_signals = get_all_signals;
window.get_all_messages = get_all_messages;
window.get_signal = get_signal;
window.get_message = get_message;
