<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getAppearance } from '$lib/stores/appearance.svelte';

  const appearance = getAppearance();

  interface Props {
    targetX?: number;
    targetY?: number;
    count?: number;
  }

  let { targetX = 64, targetY = 64, count = 5 }: Props = $props();

  const CELL_SIZE = 40;

  let canvas: HTMLCanvasElement;
  let animId = 0;
  let ctx: CanvasRenderingContext2D | null = null;
  let width = 0;
  let height = 0;
  let isVisible = true;
  let lastFrameTime = 0;
  const TARGET_FRAME_MS = 1000 / 30; // 30 FPS throttle to save memory & GPU

  interface Streamer {
    id: number;
    x: number;
    y: number;
    dirX: number; // -1, 0, 1
    dirY: number; // -1, 0, 1
    nextX: number;
    nextY: number;
    speed: number;
    length: number;
    history: { x: number; y: number }[];
    baseAlpha: number;
    lineWidth: number;
    fade: number; // 0 to 1
    reachedTarget: boolean;
    respawnDelay: number;
  }

  interface Spark {
    x: number;
    y: number;
    vx: number;
    vy: number;
    alpha: number;
    decay: number;
    size: number;
    color: string;
  }

  let streamers: Streamer[] = [];
  let sparks: Spark[] = [];

  const SPEEDS = [0.65, 0.9, 1.15, 1.4, 1.65];

  function getGridOffset() {
    const startX = ((targetX % CELL_SIZE) + CELL_SIZE) % CELL_SIZE;
    const startY = ((targetY % CELL_SIZE) + CELL_SIZE) % CELL_SIZE;
    return { startX, startY };
  }

  function triggerExplosion(x: number, y: number) {
    const colors = ['#e0f2fe', '#bae6fd', '#38bdf8', '#0ea5e9', '#7dd3fc', '#ffffff'];
    const sparkCount = 10 + Math.floor(Math.random() * 6);
    for (let i = 0; i < sparkCount; i++) {
      const angle = Math.random() * Math.PI * 2;
      const speed = 0.6 + Math.random() * 2.0;
      sparks.push({
        x: x + (Math.random() - 0.5) * 4,
        y: y + (Math.random() - 0.5) * 4,
        vx: Math.cos(angle) * speed,
        vy: Math.sin(angle) * speed,
        alpha: 1.0,
        decay: 0.025 + Math.random() * 0.03,
        size: 1.2 + Math.random() * 1.6,
        color: colors[Math.floor(Math.random() * colors.length)]
      });
    }
  }

  function chooseNextDirection(s: Streamer) {
    const { startX, startY } = getGridOffset();
    const currDist = Math.hypot(targetX - s.x, targetY - s.y);

    const candidates = [
      { dirX: s.dirX, dirY: s.dirY, straight: true },
      { dirX: s.dirY, dirY: -s.dirX, straight: false },
      { dirX: -s.dirY, dirY: s.dirX, straight: false }
    ];

    const valid = candidates.filter((c) => {
      const nx = s.x + c.dirX * CELL_SIZE;
      const ny = s.y + c.dirY * CELL_SIZE;
      return (
        nx >= startX - CELL_SIZE &&
        ny >= startY - CELL_SIZE &&
        nx <= width + CELL_SIZE * 2 &&
        ny <= height + CELL_SIZE * 2
      );
    });

    const closer = valid.filter((c) => {
      const nx = s.x + c.dirX * CELL_SIZE;
      const ny = s.y + c.dirY * CELL_SIZE;
      return Math.hypot(targetX - nx, targetY - ny) < currDist;
    });

    let chosen: { dirX: number; dirY: number };

    if (closer.length > 0) {
      const straightCloser = closer.find((c) => c.straight);
      if (straightCloser && Math.random() < 0.7) {
        chosen = straightCloser;
      } else {
        chosen = closer[Math.floor(Math.random() * closer.length)];
      }
    } else if (valid.length > 0) {
      chosen = valid[Math.floor(Math.random() * valid.length)];
    } else {
      chosen = { dirX: -s.dirX, dirY: -s.dirY };
    }

    s.dirX = chosen.dirX;
    s.dirY = chosen.dirY;
  }

  function stepStreamer(s: Streamer) {
    if (s.respawnDelay > 0) {
      s.respawnDelay--;
      return;
    }

    if (!s.reachedTarget && s.fade < 1) {
      s.fade = Math.min(1, s.fade + 0.03);
    }

    const distToTarget = Math.hypot(s.x - targetX, s.y - targetY);
    if (distToTarget <= 36 && !s.reachedTarget) {
      s.reachedTarget = true;
      triggerExplosion(s.x, s.y);
      s.fade = 0;
      s.history = [];
      return;
    }

    let remaining = s.speed;

    while (remaining > 0) {
      const distToNext =
        s.dirX !== 0 ? Math.abs(s.nextX - s.x) : Math.abs(s.nextY - s.y);

      if (remaining < distToNext) {
        s.x += s.dirX * remaining;
        s.y += s.dirY * remaining;
        remaining = 0;
      } else {
        s.x = s.nextX;
        s.y = s.nextY;
        remaining -= distToNext;

        s.history.unshift({ x: s.x, y: s.y });

        if (Math.hypot(s.x - targetX, s.y - targetY) <= 36) {
          s.reachedTarget = true;
          triggerExplosion(s.x, s.y);
          s.fade = 0;
          s.history = [];
          return;
        }

        chooseNextDirection(s);
        s.nextX = s.x + s.dirX * CELL_SIZE;
        s.nextY = s.y + s.dirY * CELL_SIZE;
      }
    }

    s.history.unshift({ x: s.x, y: s.y });
    while (s.history.length > s.length) {
      s.history.pop();
    }
  }

  function createStreamer(id: number, randomStart = false): Streamer {
    const { startX, startY } = getGridOffset();
    const cols = Math.max(1, Math.floor((width || 600) / CELL_SIZE));
    const rows = Math.max(1, Math.floor((height || 800) / CELL_SIZE));

    let x = 0;
    let y = 0;
    let dirX = 0;
    let dirY = 0;

    if (randomStart) {
      const col = 1 + Math.floor(Math.random() * (cols - 1));
      const row = 1 + Math.floor(Math.random() * (rows - 1));
      x = startX + col * CELL_SIZE;
      y = startY + row * CELL_SIZE;

      if (Math.hypot(x - targetX, y - targetY) < 100) {
        x += 3 * CELL_SIZE;
        y += 3 * CELL_SIZE;
      }

      if (Math.random() < 0.5) {
        dirX = targetX < x ? -1 : 1;
        dirY = 0;
      } else {
        dirX = 0;
        dirY = targetY < y ? -1 : 1;
      }
    } else {
      const edge = Math.random();
      if (edge < 0.5) {
        const col = Math.floor(Math.random() * (cols + 1));
        x = startX + col * CELL_SIZE;
        y = startY + (rows + 1) * CELL_SIZE;
        dirX = 0;
        dirY = -1;
      } else {
        const row = Math.floor(Math.random() * (rows + 1));
        x = startX + (cols + 1) * CELL_SIZE;
        y = startY + row * CELL_SIZE;
        dirX = -1;
        dirY = 0;
      }
    }

    const speed = SPEEDS[id % SPEEDS.length];
    const length = 22 + (id % 3) * 6;
    const baseAlpha = 0.4 + ((id * 0.12) % 0.35);
    const lineWidth = 1.3 + (id % 2) * 0.4;

    const streamer: Streamer = {
      id,
      x,
      y,
      dirX,
      dirY,
      nextX: x + dirX * CELL_SIZE,
      nextY: y + dirY * CELL_SIZE,
      speed,
      length,
      history: [{ x, y }],
      baseAlpha,
      lineWidth,
      fade: randomStart ? 0.6 + Math.random() * 0.4 : 0,
      reachedTarget: false,
      respawnDelay: randomStart ? 0 : Math.floor(Math.random() * 40)
    };

    if (randomStart) {
      const preWarmSteps = Math.floor(length * 0.6);
      for (let step = 0; step < preWarmSteps; step++) {
        if (Math.hypot(streamer.x - targetX, streamer.y - targetY) <= 36) break;
        stepStreamer(streamer);
      }
    }

    return streamer;
  }

  function initStreamers() {
    streamers = [];
    sparks = [];
    const activeCount = Math.min(count || 5, 5);
    for (let i = 0; i < activeCount; i++) {
      streamers.push(createStreamer(i, true));
    }
  }

  function resize() {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    width = rect.width;
    height = rect.height;

    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);

    if (ctx) {
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    }
  }

  function drawGrid() {
    if (!ctx || !width || !height) return;
    const { startX, startY } = getGridOffset();

    ctx.save();
    ctx.beginPath();
    ctx.strokeStyle = 'rgba(56, 189, 248, 0.07)';
    ctx.lineWidth = 1;

    for (let x = startX; x < width; x += CELL_SIZE) {
      const px = Math.floor(x) + 0.5;
      ctx.moveTo(px, 0);
      ctx.lineTo(px, height);
    }
    for (let y = startY; y < height; y += CELL_SIZE) {
      const py = Math.floor(y) + 0.5;
      ctx.moveTo(0, py);
      ctx.lineTo(width, py);
    }
    ctx.stroke();

    ctx.fillStyle = 'rgba(56, 189, 248, 0.12)';
    for (let x = startX; x < width; x += CELL_SIZE * 2) {
      for (let y = startY; y < height; y += CELL_SIZE * 2) {
        ctx.fillRect(Math.floor(x) - 1, Math.floor(y) - 1, 2, 2);
      }
    }

    const glow = ctx.createRadialGradient(targetX, targetY, 0, targetX, targetY, 60);
    glow.addColorStop(0, 'rgba(56, 189, 248, 0.14)');
    glow.addColorStop(1, 'rgba(56, 189, 248, 0)');
    ctx.fillStyle = glow;
    ctx.beginPath();
    ctx.arc(targetX, targetY, 60, 0, Math.PI * 2);
    ctx.fill();

    ctx.restore();
  }

  function updateAndDraw(timestamp: number = performance.now()) {
  	if (!isVisible) return;
  	if (timestamp - lastFrameTime < TARGET_FRAME_MS) {
  		animId = requestAnimationFrame(updateAndDraw);
  		return;
  	}
  	lastFrameTime = timestamp;

  	if (!ctx || !width || !height) {
  		if (isVisible) animId = requestAnimationFrame(updateAndDraw);
  		return;
  	}

  	ctx.clearRect(0, 0, width, height);
  	drawGrid();

  	// 1. Draw and update Streamers
  	for (let i = 0; i < streamers.length; i++) {
  		const s = streamers[i];

  		if (
  			(s.reachedTarget && s.fade <= 0) ||
  			s.x < -100 ||
  			s.y < -100 ||
  			s.x > width + 100 ||
  			s.y > height + 100
  		) {
  			streamers[i] = createStreamer(s.id, false);
  			continue;
  		}

  		stepStreamer(s);

  		if (s.history.length < 2 || s.fade <= 0) continue;

  		const head = s.history[0];
  		const tail = s.history[s.history.length - 1];
  		const gDist = Math.hypot(head.x - tail.x, head.y - tail.y);

  		let grad: CanvasGradient;
  		if (gDist > 4) {
  			grad = ctx.createLinearGradient(head.x, head.y, tail.x, tail.y);
  		} else {
  			grad = ctx.createLinearGradient(
  				head.x,
  				head.y,
  				head.x + (s.dirX !== 0 ? s.dirX : 1) * 24,
  				head.y + (s.dirY !== 0 ? s.dirY : 1) * 24
  			);
  		}

  		const alpha = s.baseAlpha * Math.max(0, s.fade);
  		grad.addColorStop(0, `rgba(224, 242, 254, ${alpha.toFixed(3)})`);
  		grad.addColorStop(0.3, `rgba(56, 189, 248, ${(alpha * 0.85).toFixed(3)})`);
  		grad.addColorStop(0.7, `rgba(14, 165, 233, ${(alpha * 0.35).toFixed(3)})`);
  		grad.addColorStop(1, 'rgba(56, 189, 248, 0)');

  		ctx.save();
  		ctx.beginPath();
  		ctx.moveTo(head.x, head.y);
  		for (let j = 1; j < s.history.length; j++) {
  			ctx.lineTo(s.history[j].x, s.history[j].y);
  		}

  		ctx.strokeStyle = grad;
  		ctx.lineWidth = s.lineWidth;
  		ctx.lineCap = 'round';
  		ctx.lineJoin = 'round';
  		ctx.stroke();

  		const headAlpha = alpha * 0.95;
  		ctx.beginPath();
  		ctx.arc(head.x, head.y, Math.max(s.lineWidth * 0.9, 1.6), 0, Math.PI * 2);
  		ctx.fillStyle = `rgba(224, 242, 254, ${headAlpha.toFixed(3)})`;
  		ctx.fill();

  		ctx.restore();
  	}

  	// 2. Draw and update Explosion Sparks
  	for (let i = sparks.length - 1; i >= 0; i--) {
  		const p = sparks[i];
  		p.x += p.vx;
  		p.y += p.vy;
  		p.vx *= 0.94;
  		p.vy *= 0.94;
  		p.alpha -= p.decay;

  		if (p.alpha <= 0) {
  			sparks.splice(i, 1);
  			continue;
  		}

  		ctx.save();
  		ctx.beginPath();
  		ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
  		ctx.fillStyle = p.color;
  		ctx.globalAlpha = Math.max(0, p.alpha);
  		ctx.fill();
  		ctx.restore();
  	}

  	if (isVisible) {
  		animId = requestAnimationFrame(updateAndDraw);
  	}
  }

  function handleVisibility(visible: boolean) {
  	if (!visible) {
  		isVisible = false;
  		cancelAnimationFrame(animId);
  	} else {
  		if (!isVisible) {
  			isVisible = true;
  			lastFrameTime = performance.now();
  			animId = requestAnimationFrame(updateAndDraw);
  		}
  	}
  }

  const onVisibilityChange = () => handleVisibility(!document.hidden);
  const onBlur = () => handleVisibility(false);
  const onFocus = () => handleVisibility(true);

  onMount(() => {
  	if (appearance.reduceMotion) return;

  	ctx = canvas.getContext('2d');
  	resize();
  	initStreamers();
  	lastFrameTime = performance.now();
  	animId = requestAnimationFrame(updateAndDraw);

  	const ro = new ResizeObserver(() => {
  		resize();
  	});
  	ro.observe(canvas);
  	document.addEventListener('visibilitychange', onVisibilityChange);
  	window.addEventListener('blur', onBlur);
  	window.addEventListener('focus', onFocus);

  	return () => {
  		cancelAnimationFrame(animId);
  		ro.disconnect();
  		document.removeEventListener('visibilitychange', onVisibilityChange);
  		window.removeEventListener('blur', onBlur);
  		window.removeEventListener('focus', onFocus);
  	};
  });

  onDestroy(() => {
    if (typeof cancelAnimationFrame !== 'undefined' && animId) {
      cancelAnimationFrame(animId);
    }
  });
</script>

{#if appearance.reduceMotion}
  <!-- Static CSS fallback: no canvas element at all, so no GPU-composited layer. -->
  <div
    class="absolute inset-0 pointer-events-none w-full h-full z-0"
    style="background: radial-gradient(circle at {targetX}px {targetY}px, rgba(56,189,248,0.12), transparent 55%)"
    aria-hidden="true"
  ></div>
{:else}
  <canvas
    bind:this={canvas}
    class="absolute inset-0 pointer-events-none w-full h-full z-0"
    aria-hidden="true"
  ></canvas>
{/if}
