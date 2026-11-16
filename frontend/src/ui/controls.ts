/** Custom UI controls — no native <select>, <input type=range>, etc. */

import { icons } from "./icons";

export type Option = { value: string; label: string };

export function customSelectHtml(id: string, label: string, options: Option[], value: string): string {
  const current = options.find((o) => o.value === value)?.label ?? options[0]?.label ?? "";
  return `
    <div class="field">
      <span class="field-label" id="${id}-label">${label}</span>
      <div class="c-select" data-select="${id}" data-value="${value}">
        <button type="button" class="c-select-trigger" aria-haspopup="listbox" aria-expanded="false" aria-labelledby="${id}-label">
          <span class="c-select-value">${current}</span>
          <span class="c-select-chevron">${icons.chevron}</span>
        </button>
        <ul class="c-select-menu" role="listbox" hidden>
          ${options
            .map(
              (o) => `
            <li role="option" data-value="${o.value}" aria-selected="${o.value === value}">
              <button type="button" class="c-select-option">${o.label}</button>
            </li>`
            )
            .join("")}
        </ul>
      </div>
    </div>`;
}

let selectCloserBound = false;

function closeAllSelects() {
  document.querySelectorAll<HTMLElement>("[data-select].open").forEach((wrap) => {
    wrap.classList.remove("open");
    const menu = wrap.querySelector<HTMLElement>(".c-select-menu");
    if (menu) menu.hidden = true;
    wrap.querySelector(".c-select-trigger")?.setAttribute("aria-expanded", "false");
  });
}

export function bindSelects(root: HTMLElement, onChange: (id: string, value: string) => void) {
  if (!selectCloserBound) {
    document.addEventListener("click", () => closeAllSelects());
    selectCloserBound = true;
  }

  root.querySelectorAll<HTMLElement>("[data-select]").forEach((wrap) => {
    const id = wrap.dataset.select!;
    const trigger = wrap.querySelector<HTMLButtonElement>(".c-select-trigger")!;
    const menu = wrap.querySelector<HTMLElement>(".c-select-menu")!;
    const valueEl = wrap.querySelector<HTMLElement>(".c-select-value")!;

    const close = () => {
      menu.hidden = true;
      trigger.setAttribute("aria-expanded", "false");
      wrap.classList.remove("open");
    };
    const open = () => {
      closeAllSelects();
      menu.hidden = false;
      trigger.setAttribute("aria-expanded", "true");
      wrap.classList.add("open");
    };

    trigger.addEventListener("click", (e) => {
      e.stopPropagation();
      if (menu.hidden) open();
      else close();
    });

    menu.querySelectorAll<HTMLElement>("[data-value]").forEach((opt) => {
      opt.addEventListener("click", (e) => {
        e.stopPropagation();
        const value = opt.dataset.value!;
        wrap.dataset.value = value;
        valueEl.textContent = opt.querySelector("button")?.textContent ?? value;
        menu.querySelectorAll("[aria-selected]").forEach((n) => n.setAttribute("aria-selected", "false"));
        opt.setAttribute("aria-selected", "true");
        close();
        onChange(id, value);
      });
    });
  });
}

export function customSliderHtml(
  id: string,
  label: string,
  min: number,
  max: number,
  step: number,
  value: number,
  display: string
): string {
  const pct = ((value - min) / (max - min)) * 100;
  return `
    <div class="field">
      <div class="field-row">
        <span class="field-label" id="${id}-label">${label}</span>
        <strong class="field-value" id="${id}-display">${display}</strong>
      </div>
      <div class="c-slider" data-slider="${id}" data-min="${min}" data-max="${max}" data-step="${step}" data-value="${value}" role="slider" tabindex="0" aria-valuemin="${min}" aria-valuemax="${max}" aria-valuenow="${value}" aria-labelledby="${id}-label">
        <div class="c-slider-track">
          <div class="c-slider-fill" style="width:${pct}%"></div>
          <div class="c-slider-thumb" style="left:${pct}%"></div>
        </div>
      </div>
    </div>`;
}

export function bindSliders(root: HTMLElement, onChange: (id: string, value: number) => void) {
  root.querySelectorAll<HTMLElement>("[data-slider]").forEach((el) => {
    const id = el.dataset.slider!;
    const min = Number(el.dataset.min);
    const max = Number(el.dataset.max);
    const step = Number(el.dataset.step) || 1;
    const fill = el.querySelector<HTMLElement>(".c-slider-fill")!;
    const thumb = el.querySelector<HTMLElement>(".c-slider-thumb")!;
    const display = root.querySelector<HTMLElement>(`#${id}-display`);

    const setValue = (raw: number, emit: boolean) => {
      const clamped = Math.min(max, Math.max(min, Math.round(raw / step) * step));
      el.dataset.value = String(clamped);
      el.setAttribute("aria-valuenow", String(clamped));
      const pct = ((clamped - min) / (max - min)) * 100;
      fill.style.width = `${pct}%`;
      thumb.style.left = `${pct}%`;
      if (display) display.textContent = String(clamped);
      if (emit) onChange(id, clamped);
    };

    const fromPointer = (clientX: number) => {
      const rect = el.querySelector(".c-slider-track")!.getBoundingClientRect();
      const t = (clientX - rect.left) / rect.width;
      setValue(min + t * (max - min), true);
    };

    let dragging = false;
    el.addEventListener("pointerdown", (e) => {
      dragging = true;
      el.setPointerCapture(e.pointerId);
      fromPointer(e.clientX);
    });
    el.addEventListener("pointermove", (e) => {
      if (!dragging) return;
      fromPointer(e.clientX);
    });
    el.addEventListener("pointerup", () => {
      dragging = false;
    });
    el.addEventListener("keydown", (e) => {
      const cur = Number(el.dataset.value);
      if (e.key === "ArrowRight" || e.key === "ArrowUp") {
        e.preventDefault();
        setValue(cur + step, true);
      } else if (e.key === "ArrowLeft" || e.key === "ArrowDown") {
        e.preventDefault();
        setValue(cur - step, true);
      }
    });
  });
}

export function segmentedHtml(
  id: string,
  label: string,
  options: Option[],
  value: string
): string {
  return `
    <div class="field">
      <span class="field-label">${label}</span>
      <div class="c-segmented" data-segmented="${id}" role="radiogroup" aria-label="${label}">
        ${options
          .map(
            (o) => `
          <button type="button" class="c-segment" role="radio" data-value="${o.value}" aria-checked="${
            o.value === value
          }">${o.label}</button>`
          )
          .join("")}
      </div>
    </div>`;
}

export function bindSegmented(root: HTMLElement, onChange: (id: string, value: string) => void) {
  root.querySelectorAll<HTMLElement>("[data-segmented]").forEach((group) => {
    const id = group.dataset.segmented!;
    group.querySelectorAll<HTMLButtonElement>(".c-segment").forEach((btn) => {
      btn.addEventListener("click", () => {
        group.querySelectorAll(".c-segment").forEach((b) => b.setAttribute("aria-checked", "false"));
        btn.setAttribute("aria-checked", "true");
        onChange(id, btn.dataset.value!);
      });
    });
  });
}

export function toggleHtml(id: string, label: string, checked: boolean): string {
  return `
    <label class="c-toggle" data-toggle="${id}">
      <button type="button" class="c-toggle-track" role="switch" aria-checked="${checked}" id="${id}">
        <span class="c-toggle-thumb"></span>
      </button>
      <span class="c-toggle-label">${label}</span>
    </label>`;
}

export function bindToggles(root: HTMLElement, onChange: (id: string, checked: boolean) => void) {
  root.querySelectorAll<HTMLElement>("[data-toggle]").forEach((wrap) => {
    const id = wrap.dataset.toggle!;
    const btn = wrap.querySelector<HTMLButtonElement>(".c-toggle-track")!;
    btn.addEventListener("click", () => {
      const next = btn.getAttribute("aria-checked") !== "true";
      btn.setAttribute("aria-checked", String(next));
      onChange(id, next);
    });
  });
}

export function readSelect(root: HTMLElement, id: string): string | null {
  return root.querySelector<HTMLElement>(`[data-select="${id}"]`)?.dataset.value ?? null;
}

export function readSlider(root: HTMLElement, id: string): number | null {
  const v = root.querySelector<HTMLElement>(`[data-slider="${id}"]`)?.dataset.value;
  return v != null ? Number(v) : null;
}
