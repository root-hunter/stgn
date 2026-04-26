// I file WASM generati da wasm-pack vengono messi in pkg/
// Esegui: wasm-pack build stng-wasm --target web --out-dir ../docs/pkg
import init, { encode_string, encode_string_secure, decode_string, decode_string_secure, encode_max_capacity } from "./pkg/stng_wasm.js";

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

  // Show/hide password toggle
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
    if (keyInput) {
      keyInput.value = "";
      keyInput.type = "password";
    }
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

// ── Encode ────────────────────────────────────────────────────────────────────

const encodeMessageInput = document.getElementById("encode-message");
const encodeCharCount    = document.getElementById("encode-char-count");
const encodeBtn          = document.getElementById("encode-btn");
const encodeResetBtn     = document.getElementById("encode-reset-btn");
const encodeResult       = document.getElementById("encode-result");
const encodeDownload     = document.getElementById("encode-download");
const encodeOutputCanvas = document.getElementById("encode-output-preview");
const capacityBar        = document.getElementById("capacity-bar");
const capacityText       = document.getElementById("capacity-text");
const capacityFill       = document.getElementById("capacity-fill");
const capacitySub        = document.getElementById("capacity-sub");

let encodeFile    = null;
let maxCapacity   = 0;

function formatBytes(n) {
  if (n >= 1024 * 1024) return (n / (1024 * 1024)).toFixed(2) + " MB";
  if (n >= 1024)        return (n / 1024).toFixed(1) + " KB";
  return n + " B";
}

function updateCapacityBar() {
  if (!maxCapacity) return;
  const used    = new TextEncoder().encode(encodeMessageInput.value).length;
  const pct     = Math.min(used / maxCapacity * 100, 100);
  const free    = Math.max(maxCapacity - used, 0);

  capacityFill.style.width = pct + "%";
  capacityFill.classList.toggle("warn", pct >= 70 && pct < 90);
  capacityFill.classList.toggle("full", pct >= 90);
  capacitySub.textContent  = `${formatBytes(used)} used · ${formatBytes(free)} free`;

  encodeCharCount.textContent = `${used} / ${maxCapacity} bytes`;
  encodeCharCount.style.color = pct >= 90 ? "var(--error)" : pct >= 70 ? "var(--warning)" : "";
}

function resetEncodeForm(clearDropzone = false) {
  encodeFile = null;
  maxCapacity = 0;
  encodeMessageInput.value = "";
  encodeCharCount.textContent = "0 characters";
  encodeCharCount.style.color = "";
  capacityBar.hidden = true;
  capacityFill.style.width = "0%";
  capacityFill.classList.remove("warn", "full");
  capacitySub.textContent = "";
  capacityText.textContent = "";
  encodeResult.hidden = true;
  encodeResetBtn.disabled = true;
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
  try {
    const bytes   = await fileToBytes(file);
    maxCapacity   = encode_max_capacity(bytes);
    capacityText.textContent = formatBytes(maxCapacity);
    capacityBar.hidden = false;
    updateCapacityBar();
  } catch { /* ignore capacity errors */ }
  updateEncodeBtn();
});

encodeMessageInput.addEventListener("input", () => {
  updateCapacityBar();
  updateEncodeBtn();
});

encodeResetBtn.addEventListener("click", () => resetEncodeForm(true));

function updateEncodeBtn() {
  encodeBtn.disabled = !encodeFile || encodeMessageInput.value.trim() === "";
}

encodeBtn.addEventListener("click", async () => {
  try {
    encodeBtn.disabled = true;
    encodeBtn.innerHTML = `<span class="btn-icon">⏳</span> Encoding…`;

    const imageBytes = await fileToBytes(encodeFile);
    const enc        = encodeEncrypt.getEncryption();
    const key        = encodeEncrypt.getKeyBytes();
    const result     = enc === "none"
      ? encode_string(imageBytes, encodeMessageInput.value)
      : encode_string_secure(imageBytes, encodeMessageInput.value, enc, key);
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

const decodeBtn      = document.getElementById("decode-btn");
const decodeResetBtn = document.getElementById("decode-reset-btn");
const decodeResult   = document.getElementById("decode-result");
const decodeOutput   = document.getElementById("decode-output");
const copyBtn        = document.getElementById("copy-btn");

let decodeFile = null;

function resetDecodeForm(clearDropzone = false) {
  decodeFile = null;
  decodeOutput.textContent = "";
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
    const message    = enc === "none"
      ? decode_string(imageBytes)
      : decode_string_secure(imageBytes, enc, key);

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
