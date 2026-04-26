// ── Service Worker registration for PWA ──
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('service-worker.js');
  });
}
// ── Utility: Suffix for filenames ──
function getDateSuffix() {
  const d = new Date();
  return `${d.getFullYear()}${String(d.getMonth()+1).padStart(2,'0')}${String(d.getDate()).padStart(2,'0')}_${String(d.getHours()).padStart(2,'0')}${String(d.getMinutes()).padStart(2,'0')}${String(d.getSeconds()).padStart(2,'0')}`;
}

// ── Download decoded as JSON ──
document.getElementById("decode-download-json").addEventListener("click", () => {
  const suffix = getDateSuffix();
  // Recupera i dati attualmente visualizzati
  const cards = decodeEntries.querySelectorAll('.decoded-card');
  if (!cards.length) return showError("No decoded entries to export");
  // Ricostruisci l'array come in renderDecodeEntries
  const entries = [];
  cards.forEach(card => {
    const name = card.querySelector('.decoded-card-name')?.textContent || "";
    if (card.querySelector('.entry-type-badge.text')) {
      // Text
      const value = card.querySelector('.decoded-text')?.textContent || "";
      entries.push({ name, type: "text", value });
    }
  });
  const blob = new Blob([JSON.stringify(entries, null, 2)], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `stng_decoded_${suffix}.json`;
  document.body.appendChild(a);
  a.click();
  setTimeout(() => {
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }, 100);
});
// I file WASM generati da wasm-pack vengono messi in pkg/
// Esegui: wasm-pack build stng-wasm --target web --out-dir ../docs/pkg
import init, {
  encode_payload, decode_payload,
  encode_max_capacity, encode_payload_size,
  zip_encoded_image,
} from "./pkg/stng_wasm.js";
// ── Download as ZIP (WASM, no external deps) ──
document.getElementById("encode-download-zip").addEventListener("click", async () => {
  try {
    const suffix = getDateSuffix();
    // Recupera i byte dell'immagine codificata
    const url = encodeDownload.href;
    if (!url) return showError("No encoded image to download");
    const response = await fetch(url);
    const imgBlob = await response.blob();
    const imgBytes = new Uint8Array(await imgBlob.arrayBuffer());
    // Crea ZIP via WASM
    const zipBytes = zip_encoded_image(imgBytes, `stng_encoded_${suffix}.png`);
    const zipBlob = new Blob([zipBytes], { type: "application/zip" });
    const zipUrl = URL.createObjectURL(zipBlob);
    const a = document.createElement("a");
    a.href = zipUrl;
    a.download = `stng_encoded_${suffix}.zip`;
    document.body.appendChild(a);
    a.click();
    setTimeout(() => {
      document.body.removeChild(a);
      URL.revokeObjectURL(zipUrl);
    }, 100);
  } catch (err) {
    showError("ZIP download failed: " + (err?.toString() ?? "Unknown error"));
  }
});

// ── Utility ──────────────────────────────────────────────────────────────────

function showToast(id, msg, duration = 4000) {
  const el = document.getElementById(id);
  el.textContent = msg;
  el.hidden = false;
  clearTimeout(el._timer);
  el._timer = setTimeout(() => (el.hidden = true), duration);
}

const showError   = (msg) => showToast("error-toast",   "❌ " + msg);
const showSuccess = (msg) => showToast("success-toast", "✅ " + msg);

async function fileToBytes(file) {
  return new Uint8Array(await file.arrayBuffer());
}

function bytesToObjectURL(bytes, mime = "image/png") {
  return URL.createObjectURL(new Blob([bytes], { type: mime }));
}

function drawOnCanvas(canvas, src) {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => {
      canvas.width  = img.naturalWidth;
      canvas.height = img.naturalHeight;
      canvas.getContext("2d").drawImage(img, 0, 0);
      canvas.style.display = "block";
      resolve();
    };
    img.src = src;
  });
}

function formatBytes(n) {
  if (n >= 1024 * 1024) return (n / (1024 * 1024)).toFixed(2) + " MB";
  if (n >= 1024)        return (n / 1024).toFixed(1) + " KB";
  return n + " B";
}

// Base64 helpers (browser-native)
function bytesToBase64(bytes) {
  let bin = "";
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin);
}

function base64ToBytes(b64) {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

// ── Tabs ──────────────────────────────────────────────────────────────────────

document.querySelectorAll(".tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    document.querySelectorAll(".tab").forEach(t => t.classList.remove("active"));
    document.querySelectorAll(".panel").forEach(p => p.classList.remove("active"));
    tab.classList.add("active");
    document.getElementById(`${tab.dataset.tab}-section`).classList.add("active");
  });
});

// ── Dropzone helper ───────────────────────────────────────────────────────────

function setupDropzone(dropzoneId, inputId, canvasId, onFile) {
  const zone   = document.getElementById(dropzoneId);
  const input  = document.getElementById(inputId);
  const canvas = document.getElementById(canvasId);

  async function handleFile(file) {
    if (!file) return;
    const url = URL.createObjectURL(file);
    await drawOnCanvas(canvas, url);
    zone.classList.add("has-image");
    onFile(file);
  }

  input.addEventListener("change", (e) => handleFile(e.target.files[0]));
  zone.addEventListener("dragover", (e) => { e.preventDefault(); zone.classList.add("drag-over"); });
  zone.addEventListener("dragleave", () => zone.classList.remove("drag-over"));
  zone.addEventListener("drop", (e) => {
    e.preventDefault();
    zone.classList.remove("drag-over");
    handleFile(e.dataTransfer.files[0]);
  });
}

// ── Encryption tabs helper ────────────────────────────────────────────────────

function setupEncryptTabs(sectionId, keyFieldId) {
  const section  = document.getElementById(sectionId);
  const keyField = document.getElementById(keyFieldId);
  let selected   = "none";

  section.querySelectorAll(".etab").forEach((btn) => {
    btn.addEventListener("click", () => {
      section.querySelectorAll(".etab").forEach(b => b.classList.remove("active"));
      btn.classList.add("active");
      selected = btn.dataset.enc;
      keyField.hidden = selected === "none";
    });
  });

  section.querySelectorAll(".key-toggle").forEach((btn) => {
    btn.addEventListener("click", () => {
      const input = document.getElementById(btn.dataset.target);
      input.type = input.type === "password" ? "text" : "password";
      btn.textContent = input.type === "password" ? "👁" : "🙈";
    });
  });

  function reset() {
    section.querySelectorAll(".etab").forEach(b => b.classList.remove("active"));
    const noneBtn = section.querySelector(".etab[data-enc='none']");
    if (noneBtn) noneBtn.classList.add("active");
    selected = "none";
    keyField.hidden = true;
    const keyInput = document.getElementById(keyFieldId.replace("-field", ""));
    if (keyInput) { keyInput.value = ""; keyInput.type = "password"; }
    const toggle = section.querySelector(".key-toggle");
    if (toggle) toggle.textContent = "👁";
  }

  return {
    getEncryption: () => selected,
    getKeyBytes: () => {
      const input = document.getElementById(keyFieldId.replace("-field", ""));
      return new TextEncoder().encode(input?.value ?? "");
    },
    reset,
  };
}

// ── Init WASM ─────────────────────────────────────────────────────────────────

await init();

const encodeEncrypt = setupEncryptTabs("encode-section", "encode-key-field");
const decodeEncrypt = setupEncryptTabs("decode-section", "decode-key-field");

// ── Encode — entry list ───────────────────────────────────────────────────────

const entriesList   = document.getElementById("entries-list");
const entriesCount  = document.getElementById("entries-count");
const addTextBtn    = document.getElementById("add-text-btn");
const addFileBtn    = document.getElementById("add-file-btn");
const capacityBar   = document.getElementById("capacity-bar");
const capacityText  = document.getElementById("capacity-text");
const capacityFill  = document.getElementById("capacity-fill");
const capacitySub   = document.getElementById("capacity-sub");
const encodeBtn     = document.getElementById("encode-btn");
const encodeResetBtn = document.getElementById("encode-reset-btn");
const encodeResult  = document.getElementById("encode-result");
const encodeDownload = document.getElementById("encode-download");
const encodeOutputCanvas = document.getElementById("encode-output-preview");

let encodeFile  = null;
let maxCapacity = 0;
let entries     = [];   // { id, type, name, value: string (text) | File (binary) }
let nextEntryId = 0;


async function totalPayloadSize() {
  // Prepara il JSON come per encode_payload
  const payload = [];
  for (const e of entries) {
    if (e.type === "text") {
      payload.push({ name: e.name || "text", type: "text", value: e.value ?? "" });
    } else if (e.type === "binary" && e.value instanceof File) {
      const bytes = await fileToBytes(e.value);
      payload.push({ name: e.name || e.value.name, type: "binary", value: bytesToBase64(bytes) });
    }
  }
  if (payload.length === 0) return 0;
  const encryption = encodeEncrypt.getEncryption();
  const keyBytes   = encodeEncrypt.getKeyBytes();
  const compress   = document.getElementById("compression-switch").checked;
  try {
    return encode_payload_size(JSON.stringify(payload), encryption, keyBytes, compress);
  } catch {
    return 0;
  }
}

async function updateCapacityBar() {
  if (!maxCapacity) return;
  const used = await totalPayloadSize();
  const pct  = Math.min(used / maxCapacity * 100, 100);
  const free = Math.max(maxCapacity - used, 0);
  capacityFill.style.width = pct + "%";
  capacityFill.classList.toggle("warn", pct >= 70 && pct < 90);
  capacityFill.classList.toggle("full", pct >= 90);
  capacitySub.textContent = `${formatBytes(used)} used · ${formatBytes(free)} free`;
}

function updateEntriesCount() {
  entriesCount.textContent = entries.length === 1 ? "1 entry" : `${entries.length} entries`;
}

function updateEncodeBtn() {
  encodeBtn.disabled = !encodeFile || entries.length === 0;
}

function removeEntry(id) {
  entries = entries.filter(e => e.id !== id);
  document.getElementById(`entry-card-${id}`)?.remove();
  updateEntriesCount();
  updateCapacityBar();
  updateEncodeBtn();
}

function addTextEntry() {
  const id = nextEntryId++;
  entries.push({ id, type: "text", name: `message${entries.filter(e=>e.type==="text").length + 1}`, value: "" });

  const card = document.createElement("div");
  card.className = "entry-card";
  card.id = `entry-card-${id}`;
  card.innerHTML = `
    <div class="entry-card-header">
      <span class="entry-type-badge text">📝 Text</span>
      <input class="entry-name-input" type="text" placeholder="Entry name" value="${entries.at(-1).name}" />
      <button class="entry-remove-btn" title="Remove">✕</button>
    </div>
    <textarea class="entry-textarea" rows="4" placeholder="Write the text to hide…"></textarea>
  `;

  const nameInput = card.querySelector(".entry-name-input");
  const textarea  = card.querySelector(".entry-textarea");
  const removeBtn = card.querySelector(".entry-remove-btn");

  nameInput.addEventListener("input", () => {
    const e = entries.find(e => e.id === id);
    if (e) e.name = nameInput.value;
    updateCapacityBar();
  });
  textarea.addEventListener("input", () => {
    const e = entries.find(e => e.id === id);
    if (e) { e.value = textarea.value; updateCapacityBar(); updateEncodeBtn(); }
  });
  removeBtn.addEventListener("click", () => removeEntry(id));

  entriesList.appendChild(card);
  updateEntriesCount();
  updateEncodeBtn();
  textarea.focus();
}

function addFileEntry() {
  const id = nextEntryId++;
  entries.push({ id, type: "binary", name: "", value: null });

  const card = document.createElement("div");
  card.className = "entry-card";
  card.id = `entry-card-${id}`;
  card.innerHTML = `
    <div class="entry-card-header">
      <span class="entry-type-badge file">📎 File</span>
      <span class="entry-file-realname">(no file)</span>
      <button class="entry-remove-btn" title="Remove">✕</button>
    </div>
    <label class="entry-file-label">
      <input type="file" class="entry-file-input" />
      <span class="entry-file-text">Click to select a file…</span>
    </label>
  `;

  const realNameSpan = card.querySelector(".entry-file-realname");
  const fileInput  = card.querySelector(".entry-file-input");
  const fileText   = card.querySelector(".entry-file-text");
  const removeBtn  = card.querySelector(".entry-remove-btn");

  fileInput.addEventListener("change", () => {
    const file = fileInput.files[0];
    if (!file) return;
    const e = entries.find(e => e.id === id);
    if (e) {
      e.value = file;
      e.name = file.name; // sempre il vero nome
      realNameSpan.textContent = file.name;
      fileText.textContent = `${file.name} (${formatBytes(file.size)})`;
      updateCapacityBar();
      updateEncodeBtn();
    }
  });
  removeBtn.addEventListener("click", () => removeEntry(id));

  entriesList.appendChild(card);
  updateEntriesCount();
}

addTextBtn.addEventListener("click", addTextEntry);
addFileBtn.addEventListener("click", addFileEntry);

// ── Encode — dropzone & reset ─────────────────────────────────────────────────

function resetEncodeForm(clearDropzone = false) {
  encodeFile = null;
  maxCapacity = 0;
  entries = [];
  nextEntryId = 0;
  entriesList.innerHTML = "";
  updateEntriesCount();
  capacityBar.hidden = true;
  capacityFill.style.width = "0%";
  capacityFill.classList.remove("warn", "full");
  capacitySub.textContent = "";
  capacityText.textContent = "";
  encodeResult.hidden = true;
  encodeResetBtn.disabled = true;
  addTextBtn.disabled = true;
  addFileBtn.disabled = true;
  encodeEncrypt.reset();
  if (clearDropzone) {
    const zone = document.getElementById("encode-dropzone");
    const canvas = document.getElementById("encode-preview");
    zone.classList.remove("has-image");
    canvas.style.display = "none";
    document.getElementById("encode-image").value = "";
  }
  updateEncodeBtn();
}

setupDropzone("encode-dropzone", "encode-image", "encode-preview", async (file) => {
  resetEncodeForm();
  encodeFile = file;
  encodeResetBtn.disabled = false;
  addTextBtn.disabled = false;
  addFileBtn.disabled = false;
  try {
    const bytes = await fileToBytes(file);
    maxCapacity = encode_max_capacity(bytes);
    capacityText.textContent = formatBytes(maxCapacity);
    capacityBar.hidden = false;
    await updateCapacityBar();
  } catch { /* ignore */ }
  updateEncodeBtn();
});

encodeResetBtn.addEventListener("click", () => resetEncodeForm(true));

// Aggiorna la barra capacità anche quando cambia compressione o encryption
document.getElementById("compression-switch").addEventListener("change", updateCapacityBar);
document.querySelectorAll(".etab").forEach(btn => btn.addEventListener("click", () => setTimeout(updateCapacityBar, 10)));

// ── Encode — submit ───────────────────────────────────────────────────────────

encodeBtn.addEventListener("click", async () => {
  try {
    encodeBtn.disabled = true;
    encodeBtn.innerHTML = `<span class="btn-icon">⏳</span> Encoding…`;

    // Build JSON entries for WASM
    const payload = [];
    for (const e of entries) {
      if (e.type === "text") {
        payload.push({ name: e.name || "text", type: "text", value: e.value ?? "" });
      } else if (e.type === "binary" && e.value instanceof File) {
        const bytes = await fileToBytes(e.value);
        payload.push({ name: e.name || e.value.name, type: "binary", value: bytesToBase64(bytes) });
      }
    }

    if (payload.length === 0) throw new Error("No valid entries to encode");

    const imageBytes = await fileToBytes(encodeFile);
    const enc        = encodeEncrypt.getEncryption();
    const key        = encodeEncrypt.getKeyBytes();
    const compress   = document.getElementById("compression-switch").checked;
    const result     = encode_payload(imageBytes, JSON.stringify(payload), enc, key, compress);
    const url        = bytesToObjectURL(result);

    const suffix = getDateSuffix();
    encodeDownload.href = url;
    encodeDownload.download = `stng_encoded_${suffix}.png`;
    await drawOnCanvas(encodeOutputCanvas, url);
    encodeResult.hidden = false;
    showSuccess(`${payload.length} ${payload.length === 1 ? "entry" : "entries"} hidden in the image!` + (compress ? " (compressed)" : " (no compression)"));
  } catch (err) {
    showError(err?.toString() ?? "Unknown error");
  } finally {
    encodeBtn.disabled = false;
    encodeBtn.innerHTML = `<span class="btn-icon">🔏</span> Encode`;
  }
});

// ── Decode ────────────────────────────────────────────────────────────────────

const decodeBtn      = document.getElementById("decode-btn");
const decodeResetBtn = document.getElementById("decode-reset-btn");
const decodeResult   = document.getElementById("decode-result");
const decodeEntries  = document.getElementById("decode-entries");

let decodeFile = null;

function renderDecodeEntries(entries) {
  decodeEntries.innerHTML = "";
  for (const entry of entries) {
    const card = document.createElement("div");
    card.className = "decoded-card";

    if (entry.type === "text") {
      card.innerHTML = `
        <div class="decoded-card-header">
          <span class="entry-type-badge text">📝 Text</span>
          <span class="decoded-card-name">${escapeHtml(entry.name)}</span>
          <button class="btn btn-outline btn-sm copy-decoded-btn">📋 Copy</button>
        </div>
        <pre class="decoded-text">${escapeHtml(entry.value)}</pre>
      `;
      card.querySelector(".copy-decoded-btn").addEventListener("click", async (btn) => {
        await navigator.clipboard.writeText(entry.value);
        btn.target.textContent = "✅ Copied!";
        setTimeout(() => btn.target.textContent = "📋 Copy", 2000);
      });
    } else {
      // binary → download button
      const bytes   = base64ToBytes(entry.value);
      const mime    = guessMime(entry.name);
      const url     = bytesToObjectURL(bytes, mime);
      card.innerHTML = `
        <div class="decoded-card-header">
          <span class="entry-type-badge file">📎 File</span>
          <span class="decoded-card-name">${escapeHtml(entry.name)}</span>
          <span class="decoded-size">${formatBytes(bytes.length)}</span>
        </div>
        <a class="btn btn-success btn-sm" href="${url}" download="${escapeHtml(entry.name)}">⬇ Download</a>
      `;
    }

    decodeEntries.appendChild(card);
  }
}

function escapeHtml(s) {
  return String(s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;");
}

function guessMime(name) {
  const ext = name.split(".").pop().toLowerCase();
  const map = { png:"image/png", jpg:"image/jpeg", jpeg:"image/jpeg", gif:"image/gif",
                webp:"image/webp", pdf:"application/pdf", txt:"text/plain",
                json:"application/json", zip:"application/zip" };
  return map[ext] ?? "application/octet-stream";
}

function resetDecodeForm(clearDropzone = false) {
  decodeFile = null;
  decodeEntries.innerHTML = "";
  decodeResult.hidden = true;
  decodeBtn.disabled = true;
  decodeResetBtn.disabled = true;
  decodeEncrypt.reset();
  if (clearDropzone) {
    const zone = document.getElementById("decode-dropzone");
    const canvas = document.getElementById("decode-preview");
    zone.classList.remove("has-image");
    canvas.style.display = "none";
    document.getElementById("decode-image").value = "";
  }
}

setupDropzone("decode-dropzone", "decode-image", "decode-preview", (file) => {
  resetDecodeForm();
  decodeFile = file;
  decodeBtn.disabled = false;
  decodeResetBtn.disabled = false;
});

decodeResetBtn.addEventListener("click", () => resetDecodeForm(true));

decodeBtn.addEventListener("click", async () => {
  try {
    decodeBtn.disabled = true;
    decodeBtn.innerHTML = `<span class="btn-icon">⏳</span> Decoding…`;

    const imageBytes = await fileToBytes(decodeFile);
    const enc        = decodeEncrypt.getEncryption();
    const key        = decodeEncrypt.getKeyBytes();
    const json       = decode_payload(imageBytes, enc, key);
    const parsed     = JSON.parse(json);

    renderDecodeEntries(parsed);
    decodeResult.hidden = false;
    showSuccess(`${parsed.length} ${parsed.length === 1 ? "entry" : "entries"} extracted!`);
  } catch (err) {
    showError(err?.toString() ?? "No payload found or unsupported format");
  } finally {
    decodeBtn.disabled = false;
    decodeBtn.innerHTML = `<span class="btn-icon">🔓</span> Decode`;
  }
});

