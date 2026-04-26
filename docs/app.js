// I file WASM generati da wasm-pack vengono messi in pkg/
// Esegui: wasm-pack build stng-wasm --target web --out-dir ../docs/pkg
import init, { encode_string, decode_string } from "./pkg/stng_wasm.js";

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

// ── Init WASM ─────────────────────────────────────────────────────────────────

await init();

// ── Encode ────────────────────────────────────────────────────────────────────

const encodeMessageInput = document.getElementById("encode-message");
const encodeCharCount    = document.getElementById("encode-char-count");
const encodeBtn          = document.getElementById("encode-btn");
const encodeResult       = document.getElementById("encode-result");
const encodeDownload     = document.getElementById("encode-download");
const encodeOutputCanvas = document.getElementById("encode-output-preview");

let encodeFile = null;

setupDropzone("encode-dropzone", "encode-image", "encode-preview", (file) => {
  encodeFile = file;
  updateEncodeBtn();
});

encodeMessageInput.addEventListener("input", () => {
  encodeCharCount.textContent = `${encodeMessageInput.value.length} characters`;
  updateEncodeBtn();
});

function updateEncodeBtn() {
  encodeBtn.disabled = !encodeFile || encodeMessageInput.value.trim() === "";
}

encodeBtn.addEventListener("click", async () => {
  try {
    encodeBtn.disabled = true;
    encodeBtn.innerHTML = `<span class="btn-icon">⏳</span> Encoding…`;

    const imageBytes = await fileToBytes(encodeFile);
    const result     = encode_string(imageBytes, encodeMessageInput.value);
    const url        = bytesToObjectURL(result);

    encodeDownload.href = url;
    await drawOnCanvas(encodeOutputCanvas, url);
    encodeResult.hidden = false;
    showSuccess("Message hidden in the image!");
  } catch (err) {
    showError(err?.toString() ?? "Unknown error");
  } finally {
    encodeBtn.disabled = false;
    encodeBtn.innerHTML = `<span class="btn-icon">🔏</span> Encode`;
  }
});

// ── Decode ────────────────────────────────────────────────────────────────────

const decodeBtn    = document.getElementById("decode-btn");
const decodeResult = document.getElementById("decode-result");
const decodeOutput = document.getElementById("decode-output");
const copyBtn      = document.getElementById("copy-btn");

let decodeFile = null;

setupDropzone("decode-dropzone", "decode-image", "decode-preview", (file) => {
  decodeFile = file;
  decodeBtn.disabled = false;
});

decodeBtn.addEventListener("click", async () => {
  try {
    decodeBtn.disabled = true;
    decodeBtn.innerHTML = `<span class="btn-icon">⏳</span> Decoding…`;

    const imageBytes = await fileToBytes(decodeFile);
    const message    = decode_string(imageBytes);

    decodeOutput.textContent = message;
    decodeResult.hidden = false;
    showSuccess("Message extracted!");
  } catch (err) {
    showError(err?.toString() ?? "No message found or unsupported format");
  } finally {
    decodeBtn.disabled = false;
    decodeBtn.innerHTML = `<span class="btn-icon">🔓</span> Decode`;
  }
});

copyBtn.addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText(decodeOutput.textContent);
    copyBtn.textContent = "✅ Copied!";
    setTimeout(() => (copyBtn.textContent = "📋 Copy text"), 2000);
  } catch {
    showError("Could not copy to clipboard");
  }
});
