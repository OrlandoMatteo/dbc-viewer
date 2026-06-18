const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const pageSize = 10;

let currentPage = 1;
let searchResults = [];
let browseItems = [];
let latestWarnings = [];
let historyDepth = 0;
let historyIndex = -1;
let greetInputEl;
let greetMsgEl;

(async () => {
  await listen("file-open", async (event) => {
    const { path: filePath } = event.payload;

    try {
      const response = await invoke("load_file_from_path", { path: filePath });
      setFileStatus(response);
      resetInspector();
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

function plural(count, label) {
  return `${count} ${label}${count === 1 ? "" : "s"}`;
}

function currentAppPage() {
  return document.body.dataset.page || "search";
}

function selectedSearchKind() {
  return document.querySelector('input[name="search-kind"]:checked')?.value || "all";
}

function setFileStatus(response) {
  const filenameEl = document.getElementById("filename");
  latestWarnings = response.warnings || [];

  if (filenameEl) {
    filenameEl.textContent = response.message || "No DBC loaded";
    filenameEl.classList.remove("is-loaded", "is-empty", "has-warnings");
    filenameEl.classList.add(response.loaded ? "is-loaded" : "is-empty");
    if (response.loaded && latestWarnings.length) {
      filenameEl.classList.add("has-warnings");
    }
  }

  renderWarnings();
}

function renderWarnings() {
  const warningToggle = document.getElementById("warning-toggle");
  const warningCount = document.getElementById("warning-count");
  const warningPanel = document.getElementById("warning-panel");

  if (!warningToggle || !warningCount || !warningPanel) {
    return;
  }

  warningCount.textContent = latestWarnings.length;
  warningToggle.classList.toggle("d-none", latestWarnings.length === 0);

  warningPanel.innerHTML = latestWarnings.length
    ? `
      <div class="warning-panel-inner">
        <div class="warning-title">Parser warnings</div>
        <ul>
          ${latestWarnings.map((warning) => `<li>${escapeHtml(warning)}</li>`).join("")}
        </ul>
      </div>
    `
    : "";
}

async function refreshCurrentPage() {
  const page = currentAppPage();
  if (page === "signals") {
    await get_all_signals();
  } else if (page === "messages") {
    await get_all_messages();
  } else if (greetInputEl?.value?.trim().length > 2) {
    await greet();
  } else {
    renderSearchResults();
  }
}

function triggerFileUpload() {
  document.getElementById("file-input")?.click();
}

async function prevHist() {
  if (historyIndex <= 0) {
    updateHistoryButtons();
    return;
  }

  try {
    const view = await invoke("handle_history", { query: "Prev" });
    historyIndex -= 1;
    renderViewItem(view);
  } catch (error) {
    console.debug(error);
  } finally {
    updateHistoryButtons();
  }
}

async function nextHist() {
  if (historyIndex + 1 >= historyDepth) {
    updateHistoryButtons();
    return;
  }

  try {
    const view = await invoke("handle_history", { query: "Next" });
    historyIndex += 1;
    renderViewItem(view);
  } catch (error) {
    console.debug(error);
  } finally {
    updateHistoryButtons();
  }
}

function recordHistorySelection() {
  if (historyIndex + 1 < historyDepth) {
    historyDepth = historyIndex + 1;
  }
  historyDepth += 1;
  historyIndex = historyDepth - 1;
  updateHistoryButtons();
}

function resetHistory() {
  historyDepth = 0;
  historyIndex = -1;
  updateHistoryButtons();
}

function updateHistoryButtons() {
  const prevBtn = document.getElementById("prevHist");
  const nextBtn = document.getElementById("nextHist");
  if (prevBtn) {
    prevBtn.disabled = historyIndex <= 0;
  }
  if (nextBtn) {
    nextBtn.disabled = historyIndex < 0 || historyIndex + 1 >= historyDepth;
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

function filteredSearchResults() {
  const kind = selectedSearchKind();
  return kind === "all" ? searchResults : searchResults.filter((result) => result.kind === kind);
}

function renderSearchResults() {
  if (!greetMsgEl) {
    return;
  }

  const results = filteredSearchResults();
  const start = (currentPage - 1) * pageSize;
  const visibleResults = results.slice(start, start + pageSize);
  const countEl = document.getElementById("search-count");

  if (countEl) {
    countEl.textContent = plural(results.length, "result");
  }

  if (!greetInputEl?.value?.trim()) {
    greetMsgEl.innerHTML = renderEmptyState(
      "Search loaded DBC data",
      "Enter at least three characters to find messages and signals."
    );
  } else if (visibleResults.length === 0) {
    greetMsgEl.innerHTML = renderEmptyState("No matches", "Try a different name, CAN ID, or signal ID.");
  } else {
    greetMsgEl.innerHTML = `
      <div class="list-group result-group">
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
    nextBtn.disabled = currentPage * pageSize >= results.length;
  }
}

function renderSearchResult(result) {
  const action = result.kind === "message" ? "view-message" : "view-signal";
  const badgeText = result.kind === "message" ? "Message" : "Signal";

  return `
    <button type="button"
      class="list-group-item list-group-item-action result-item"
      data-action="${action}"
      data-name="${escapeAttr(result.name)}">
      <span>
        <span class="result-name">${escapeHtml(result.name)}</span>
        <span class="result-meta">${escapeHtml(result.id)}</span>
      </span>
      <span class="kind-badge kind-${escapeAttr(result.kind)}">${badgeText}</span>
    </button>
  `;
}

async function show_signal(name) {
  const view = await invoke("show_signal", { query: name });
  recordHistorySelection();
  renderViewItem(view);
}

async function show_message(name) {
  const view = await invoke("show_message", { query: name });
  recordHistorySelection();
  renderViewItem(view);
}

async function is_dbc_loaded() {
  const response = await invoke("is_dbc_loaded");
  setFileStatus(response);
}

async function load_dbc(file, filename) {
  const response = await invoke("upload_dbc", { base64Data: file, filename });
  setFileStatus(response);
  resetInspector();
  await refreshCurrentPage();
}

async function get_all_signals() {
  const page = document.getElementById("page");
  if (!page) {
    return;
  }

  showSpinner(page);
  browseItems = await invoke("get_all_signals");
  renderSignalsTable();
}

async function get_all_messages() {
  const page = document.getElementById("page");
  if (!page) {
    return;
  }

  showSpinner(page);
  browseItems = await invoke("get_all_messages");
  renderMessagesTable();
}

function browseFilterText() {
  return document.getElementById("browse-filter")?.value.trim().toLowerCase() || "";
}

function renderMessagesTable() {
  const page = document.getElementById("page");
  const countEl = document.getElementById("browse-count");
  if (!page) {
    return;
  }

  const filter = browseFilterText();
  const rows = browseItems.filter((detail) => {
    const message = detail.message;
    return [
      message.name,
      formatHex(message.can_id),
      message.pgn,
      message.dlc,
      detail.signals?.length,
    ]
      .join(" ")
      .toLowerCase()
      .includes(filter);
  });

  if (countEl) {
    countEl.textContent = `${rows.length} of ${plural(browseItems.length, "message")}`;
  }

  if (!browseItems.length) {
    page.innerHTML = renderEmptyState("No DBC loaded", "Open a DBC file to browse messages.");
    return;
  }

  if (!rows.length) {
    page.innerHTML = renderEmptyState("No messages match", "Clear or change the table filter.");
    return;
  }

  page.innerHTML = `
    <div class="table-responsive">
      <table class="table table-hover align-middle browse-grid">
        <thead>
          <tr>
            <th>Name</th>
            <th>CAN ID</th>
            <th>PGN</th>
            <th>DLC</th>
            <th>Signals</th>
            <th>Frame</th>
          </tr>
        </thead>
        <tbody>
          ${rows.map(renderMessageRow).join("")}
        </tbody>
      </table>
    </div>
  `;
}

function renderMessageRow(detail) {
  const message = detail.message;
  return `
    <tr class="browse-row" data-action="view-message" data-name="${escapeAttr(message.name)}">
      <td>
        <div class="row-title">${escapeHtml(message.name)}</div>
        ${message.label ? `<div class="row-caption">${escapeHtml(message.label)}</div>` : ""}
      </td>
      <td><code>${escapeHtml(formatHex(message.can_id))}</code></td>
      <td>${escapeHtml(message.pgn)}</td>
      <td>${escapeHtml(message.dlc)}</td>
      <td>${escapeHtml(detail.signals?.length || 0)}</td>
      <td>${message.isExtendedFrame ? "Extended" : "Standard"}</td>
    </tr>
  `;
}

function renderSignalsTable() {
  const page = document.getElementById("page");
  const countEl = document.getElementById("browse-count");
  if (!page) {
    return;
  }

  const filter = browseFilterText();
  const rows = browseItems.filter((signal) =>
    [
      valueOf(signal, "name"),
      valueOf(signal, "msgName", "msg_name"),
      valueOf(signal, "sig_id"),
      valueOf(signal, "sourceUnit", "source_unit"),
      valueOf(signal, "category"),
    ]
      .join(" ")
      .toLowerCase()
      .includes(filter)
  );

  if (countEl) {
    countEl.textContent = `${rows.length} of ${plural(browseItems.length, "signal")}`;
  }

  if (!browseItems.length) {
    page.innerHTML = renderEmptyState("No DBC loaded", "Open a DBC file to browse signals.");
    return;
  }

  if (!rows.length) {
    page.innerHTML = renderEmptyState("No signals match", "Clear or change the table filter.");
    return;
  }

  page.innerHTML = `
    <div class="table-responsive">
      <table class="table table-hover align-middle browse-grid">
        <thead>
          <tr>
            <th>Name</th>
            <th>Message</th>
            <th>Signal ID</th>
            <th>Start</th>
            <th>Bits</th>
            <th>Unit</th>
          </tr>
        </thead>
        <tbody>
          ${rows.map(renderSignalRow).join("")}
        </tbody>
      </table>
    </div>
  `;
}

function renderSignalRow(signal) {
  return `
    <tr class="browse-row" data-action="view-signal" data-name="${escapeAttr(valueOf(signal, "name"))}">
      <td>
        <div class="row-title">${escapeHtml(valueOf(signal, "name"))}</div>
        ${valueOf(signal, "category") ? `<div class="row-caption">${escapeHtml(valueOf(signal, "category"))}</div>` : ""}
      </td>
      <td>${escapeHtml(valueOf(signal, "msgName", "msg_name"))}</td>
      <td>${escapeHtml(valueOf(signal, "sig_id"))}</td>
      <td>${escapeHtml(valueOf(signal, "startBit", "start_bit"))}</td>
      <td>${escapeHtml(valueOf(signal, "bitLength", "bit_length"))}</td>
      <td>${escapeHtml(valueOf(signal, "sourceUnit", "source_unit"))}</td>
    </tr>
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

function resetInspector() {
  resetHistory();
  const signalCard = document.getElementById("signal_card");
  if (signalCard) {
    signalCard.innerHTML = renderEmptyState(
      "Nothing selected",
      "Choose a result or table row to inspect decoded DBC data."
    );
  }
}

function renderSignalCard(signal) {
  const msgName = valueOf(signal, "msgName", "msg_name");
  return `
    <article class="detail-card detail-signal">
      <div class="detail-heading">
        <span class="kind-badge kind-signal">Signal</span>
        <h3>${escapeHtml(valueOf(signal, "name"))}</h3>
        ${valueOf(signal, "label") ? `<p>${escapeHtml(valueOf(signal, "label"))}</p>` : ""}
      </div>
      <div class="meta-strip">
        ${renderChip("Message", msgName)}
        ${renderChip("ID", valueOf(signal, "sig_id"))}
        ${renderChip("Unit", valueOf(signal, "sourceUnit", "source_unit"))}
      </div>
      ${renderSignalDetails(signal)}
    </article>
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
    ["Interval", valueOf(signal, "interval")],
    ["Category", valueOf(signal, "category")],
    ["Msg ID", formatHex(valueOf(signal, "msgId", "msg_id"))],
  ];

  return `
    <dl class="detail-grid">
      ${fields.map(renderField).join("")}
      <div>
        <dt>Msg Name</dt>
        <dd>
          ${msgName
            ? `<button type="button" class="btn btn-link p-0 align-baseline" data-action="view-message" data-name="${escapeAttr(msgName)}">${escapeHtml(msgName)}</button>`
            : ""}
        </dd>
      </div>
    </dl>
    ${renderStatesTable(valueOf(signal, "states") || [])}
  `;
}

function renderMessageCard(detail) {
  return `
    <article class="detail-card detail-message">
      ${renderMessageDetails(detail)}
    </article>
  `;
}

function renderMessageDetails(detail) {
  const message = detail.message;
  const signals = detail.signals || [];

  return `
    <div class="detail-heading">
      <span class="kind-badge kind-message">Message</span>
      <h3>${escapeHtml(message.name)}</h3>
      ${message.label ? `<p>${escapeHtml(message.label)}</p>` : ""}
    </div>
    <div class="meta-strip">
      ${renderChip("CAN ID", formatHex(message.can_id))}
      ${renderChip("PGN", message.pgn)}
      ${renderChip("DLC", message.dlc)}
      ${renderChip("Signals", signals.length)}
    </div>
    <dl class="detail-grid">
      ${[
        ["Source", message.source],
        ["Priority", message.priority],
        ["Extended frame", message.isExtendedFrame ? "Yes" : "No"],
        ["Line", message.line_in_dbc],
      ].map(renderField).join("")}
    </dl>
    <div class="related-list">
      <h4>Signals</h4>
      ${signals.length
        ? `<div class="list-group">${signals.map((signal) => `
            <button type="button" class="list-group-item list-group-item-action related-item" data-action="view-signal" data-name="${escapeAttr(valueOf(signal, "name"))}">
              <span>${escapeHtml(valueOf(signal, "name"))}</span>
              <small>${escapeHtml(valueOf(signal, "sourceUnit", "source_unit"))}</small>
            </button>
          `).join("")}</div>`
        : '<div class="text-muted">No signal details found</div>'}
    </div>
  `;
}

function renderChip(label, value) {
  if (value === "" || value === undefined || value === null) {
    return "";
  }
  return `
    <span class="meta-chip">
      <span>${escapeHtml(label)}</span>
      <strong>${escapeHtml(value)}</strong>
    </span>
  `;
}

function renderField([label, value]) {
  return `
    <div>
      <dt>${escapeHtml(label)}</dt>
      <dd>${escapeHtml(value)}</dd>
    </div>
  `;
}

function renderStatesTable(states) {
  if (!states.length) {
    return "";
  }

  return `
    <div class="states-table">
      <h4>States</h4>
      <div class="table-responsive">
        <table class="table table-sm table-hover">
          <thead><tr><th>Value</th><th>State</th></tr></thead>
          <tbody>
            ${states.map((state) => `<tr><td>${escapeHtml(state.value)}</td><td>${escapeHtml(state.state)}</td></tr>`).join("")}
          </tbody>
        </table>
      </div>
    </div>
  `;
}

function renderEmptyState(title, message) {
  return `
    <div class="empty-state">
      <strong>${escapeHtml(title)}</strong>
      <span>${escapeHtml(message)}</span>
    </div>
  `;
}

function showSpinner(target = document.getElementById("page")) {
  if (!target) {
    return;
  }

  target.innerHTML = `
    <div class="loading-state">
      <div class="spinner-border spinner-border-sm" role="status"></div>
      <span>Loading...</span>
    </div>
  `;
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
      const results = filteredSearchResults();
      if (currentPage * pageSize < results.length) {
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
    greetInputEl.addEventListener("input", () => {
      greet();
    });
  }

  document.querySelectorAll('input[name="search-kind"]').forEach((input) => {
    input.addEventListener("change", () => {
      currentPage = 1;
      renderSearchResults();
    });
  });

  renderSearchResults();
}

function setupBrowsePage() {
  const filter = document.getElementById("browse-filter");
  if (filter) {
    filter.addEventListener("input", () => {
      if (currentAppPage() === "messages") {
        renderMessagesTable();
      } else {
        renderSignalsTable();
      }
    });
  }
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

window.addEventListener("DOMContentLoaded", async () => {
  setupFileInput();
  setupSearchPage();
  setupBrowsePage();
  resetInspector();
  await is_dbc_loaded();
  await refreshCurrentPage();
});

function get_signal(name) {
  show_signal(name);
}

function get_message(name) {
  show_message(name);
}

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
