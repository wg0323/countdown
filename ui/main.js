(function () {
  const { invoke } = window.__TAURI__.core;
  const { listen } = window.__TAURI__.event;

  const el = {
    setup: document.getElementById("setup"),
    countdown: document.getElementById("countdown"),
    inH: document.getElementById("in-h"),
    inM: document.getElementById("in-m"),
    inS: document.getElementById("in-s"),
    btnStart: document.getElementById("btn-start"),
    btnSetupClose: document.getElementById("btn-setup-close"),
    caption: document.getElementById("caption"),
    clock: document.getElementById("clock"),
    btnPause: document.getElementById("btn-pause"),
    btnCancel: document.getElementById("btn-cancel"),
    btnMin: document.getElementById("btn-min"),
    btnClose: document.getElementById("btn-close"),
  };

  const clock = window.createClock(el.clock);

  let phase = "idle";
  let lastRemainingMs = 0;

  const clamp = (value, min, max) => Math.min(max, Math.max(min, Number(value) || 0));

  function normalizeInputs() {
    el.inH.value = clamp(el.inH.value, 0, 99);
    el.inM.value = clamp(el.inM.value, 0, 59);
    el.inS.value = clamp(el.inS.value, 0, 59);
    syncStartButton();
  }

  function totalSeconds() {
    return Number(el.inH.value) * 3600 + Number(el.inM.value) * 60 + Number(el.inS.value);
  }

  function syncStartButton() {
    el.btnStart.disabled = totalSeconds() <= 0;
  }

  function pad2(value) {
    return String(value).padStart(2, "0");
  }

  function setCaption(text) {
    el.caption.textContent = text;
  }

  function captionForRemaining(remainingMs) {
    const endsAt = new Date(Date.now() + remainingMs);
    return `预计 ${pad2(endsAt.getHours())}:${pad2(endsAt.getMinutes())} 结束`;
  }

  function renderRemaining(remainingMs) {
    lastRemainingMs = remainingMs;
    clock.render(Math.ceil(remainingMs / 1000));
  }

  function showSetup() {
    el.setup.hidden = false;
    el.countdown.hidden = true;
    clock.reset();
    el.btnPause.textContent = "暂停";
  }

  function showCountdown() {
    el.setup.hidden = true;
    el.countdown.hidden = false;
  }

  function applyState(state) {
    phase = state.phase;
    lastRemainingMs = state.remaining_ms;

    if (phase === "idle") {
      showSetup();
      return;
    }

    showCountdown();
    clock.render(Math.ceil(state.remaining_ms / 1000));

    if (phase === "running") {
      el.btnPause.textContent = "暂停";
      setCaption(captionForRemaining(state.remaining_ms));
    } else if (phase === "paused") {
      el.btnPause.textContent = "继续";
      setCaption("已暂停");
    } else {
      el.btnPause.textContent = "重置";
      setCaption("时间到");
      clock.flash();
    }
  }

  async function refreshState() {
    try {
      applyState(await invoke("get_state"));
    } catch (error) {
      console.error("读取状态失败", error);
    }
  }

  async function startCountdown() {
    const seconds = totalSeconds();
    if (seconds <= 0) {
      return;
    }
    try {
      await invoke("start_countdown", { seconds });
      phase = "running";
      el.btnPause.textContent = "暂停";
      showCountdown();
      clock.reset();
      clock.render(seconds);
      setCaption(captionForRemaining(seconds * 1000));
    } catch (error) {
      console.error("开始倒计时失败", error);
    }
  }

  async function togglePause() {
    try {
      if (phase === "running") {
        await invoke("pause_countdown");
        phase = "paused";
        el.btnPause.textContent = "继续";
        setCaption("已暂停");
      } else if (phase === "paused") {
        await invoke("resume_countdown");
        phase = "running";
        el.btnPause.textContent = "暂停";
        renderRemaining(lastRemainingMs);
        setCaption(captionForRemaining(lastRemainingMs));
      } else if (phase === "finished") {
        await invoke("cancel_countdown");
        phase = "idle";
        showSetup();
      }
    } catch (error) {
      console.error("切换暂停状态失败", error);
    }
  }

  async function cancelCountdown() {
    try {
      await invoke("cancel_countdown");
      phase = "idle";
      showSetup();
    } catch (error) {
      console.error("取消倒计时失败", error);
    }
  }

  function hideWindow() {
    invoke("hide_window").catch((error) => console.error("隐藏窗口失败", error));
  }

  el.inH.addEventListener("input", normalizeInputs);
  el.inM.addEventListener("input", normalizeInputs);
  el.inS.addEventListener("input", normalizeInputs);

  document.querySelectorAll(".preset").forEach((button) => {
    button.addEventListener("click", () => {
      el.inH.value = 0;
      el.inM.value = Number(button.dataset.min);
      el.inS.value = 0;
      syncStartButton();
    });
  });

  el.btnStart.addEventListener("click", startCountdown);
  el.btnPause.addEventListener("click", togglePause);
  el.btnCancel.addEventListener("click", cancelCountdown);
  el.btnMin.addEventListener("click", hideWindow);
  el.btnClose.addEventListener("click", hideWindow);
  el.btnSetupClose.addEventListener("click", hideWindow);

  listen("countdown://tick", (event) => {
    if (phase === "running") {
      renderRemaining(event.payload.remaining_ms);
    }
  });

  listen("countdown://finished", () => {
    phase = "finished";
    clock.render(0);
    clock.flash();
    setCaption("时间到");
    el.btnPause.textContent = "重置";
  });

  listen("countdown://sync", refreshState);

  normalizeInputs();
  refreshState();
})();
