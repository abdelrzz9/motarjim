import { PIPELINE_STAGES } from '../constants.js';
import { icon } from '../utils/icons.js';

export function createPipeline() {
  const el = document.createElement('div');
  el.className = 'pipeline';
  el.setAttribute('role', 'progressbar');
  el.setAttribute('aria-label', 'Compilation pipeline progress');
  el.setAttribute('aria-valuemin', '0');
  el.setAttribute('aria-valuemax', String(PIPELINE_STAGES.length - 1));

  const stageEls = [];

  PIPELINE_STAGES.forEach((stage, i) => {
    const stageEl = document.createElement('div');
    stageEl.className = 'pipeline-stage';
    stageEl.dataset.stage = i;

    const iconEl = document.createElement('span');
    iconEl.className = 'pipeline-stage-icon';
    iconEl.innerHTML = icon(stage.icon);

    const label = document.createElement('span');
    label.className = 'pipeline-stage-label';
    label.textContent = stage.label;

    stageEl.appendChild(iconEl);
    stageEl.appendChild(label);
    el.appendChild(stageEl);
    stageEls.push(stageEl);

    if (i < PIPELINE_STAGES.length - 1) {
      const link = document.createElement('div');
      link.className = 'pipeline-link';
      link.dataset.link = i;
      el.appendChild(link);
    }
  });

  function reset() {
    stageEls.forEach(s => {
      s.classList.remove('active', 'done', 'pulse');
    });
    el.querySelectorAll('.pipeline-link').forEach(l => {
      l.classList.remove('filled', 'processing');
    });
    el.setAttribute('aria-valuenow', '0');
  }

  function setStage(index) {
    reset();
    for (let i = 0; i <= index && i < stageEls.length; i++) {
      if (i < index) {
        stageEls[i].classList.add('done');
        const link = el.querySelector(`[data-link="${i}"]`);
        if (link) link.classList.add('filled');
      } else {
        stageEls[i].classList.add('active', 'pulse');
        const link = el.querySelector(`[data-link="${i - 1}"]`);
        if (link) link.classList.add('filled');
      }
    }
    el.setAttribute('aria-valuenow', String(index + 1));
  }

  function complete() {
    stageEls.forEach(s => s.classList.add('done'));
    el.querySelectorAll('.pipeline-link').forEach(l => l.classList.add('filled'));
    el.setAttribute('aria-valuenow', String(PIPELINE_STAGES.length));
  }

  reset();

  return { el, reset, setStage, complete };
}
