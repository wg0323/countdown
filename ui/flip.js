// 翻牌时钟：3 张卡片（时 / 分 / 秒），每张两位数字，值变化时做一次 3D 翻转
(function () {
  const FLIP_MS = 520;

  function pad2(value) {
    return String(value).padStart(2, "0");
  }

  function createCard() {
    const card = document.createElement("div");
    card.className = "card";
    card.innerHTML =
      '<div class="half upper"><span class="digits"></span></div>' +
      '<div class="half lower"><span class="digits"></span></div>' +
      '<div class="flap flap-top"><span class="digits"></span></div>' +
      '<div class="flap flap-bottom"><span class="digits"></span></div>' +
      '<div class="seam"></div>';
    return card;
  }

  function setCardValue(card, next) {
    const prev = card.dataset.value;
    if (prev === next) {
      return;
    }
    card.dataset.value = next;

    const upper = card.querySelector(".half.upper .digits");
    const lower = card.querySelector(".half.lower .digits");

    if (prev === undefined) {
      upper.textContent = next;
      lower.textContent = next;
      return;
    }

    // 静态上半立即变成新值，静态下半保持旧值，由下半张翻上来替换
    upper.textContent = next;
    lower.textContent = prev;
    card.querySelector(".flap.flap-top .digits").textContent = prev;
    card.querySelector(".flap.flap-bottom .digits").textContent = next;

    card.classList.remove("flipping");
    void card.offsetWidth;
    card.classList.add("flipping");

    clearTimeout(card.flipTimer);
    card.flipTimer = setTimeout(() => {
      lower.textContent = next;
      card.classList.remove("flipping");
    }, FLIP_MS);
  }

  function createClock(root) {
    const cards = [createCard(), createCard(), createCard()];
    cards.forEach((card) => root.appendChild(card));

    function render(totalSeconds) {
      const total = Math.max(0, Math.floor(totalSeconds));
      const values = [
        pad2(Math.min(99, Math.floor(total / 3600))),
        pad2(Math.floor((total % 3600) / 60)),
        pad2(total % 60),
      ];
      cards.forEach((card, index) => setCardValue(card, values[index]));
    }

    function reset() {
      cards.forEach((card) => {
        delete card.dataset.value;
        card.classList.remove("flipping");
        card.querySelectorAll(".digits").forEach((el) => {
          el.textContent = "00";
        });
      });
      root.classList.remove("finished");
    }

    function flash() {
      root.classList.remove("finished");
      void root.offsetWidth;
      root.classList.add("finished");
    }

    reset();
    return { render, reset, flash };
  }

  window.createClock = createClock;
})();
