"""Circular void buffers for audio and visual streams (pre-allocated RAM rings)."""

from __future__ import annotations

import threading
import time
from collections import deque
from dataclasses import dataclass
from typing import Deque, Optional

import numpy as np


@dataclass
class AudioChunk:
    samples: np.ndarray
    timestamp: float
    rms: float


@dataclass
class VideoFrame:
    jpeg_bytes: bytes
    timestamp: float
    motion_score: float


class AudioVoidBuffer:
    """Ring buffer holding the last N seconds of mono float32 audio."""

    def __init__(self, sample_rate: int, seconds: float) -> None:
        self.sample_rate = sample_rate
        self.capacity = int(sample_rate * seconds)
        self._buf = np.zeros(self.capacity, dtype=np.float32)
        self._write = 0
        self._filled = 0
        self._lock = threading.Lock()

    def push(self, chunk: np.ndarray) -> float:
        flat = np.asarray(chunk, dtype=np.float32).flatten()
        rms = float(np.sqrt(np.mean(flat**2)) if flat.size else 0.0)
        with self._lock:
            n = flat.size
            if n >= self.capacity:
                self._buf[:] = flat[-self.capacity :]
                self._write = 0
                self._filled = self.capacity
            else:
                end = self._write + n
                if end <= self.capacity:
                    self._buf[self._write : end] = flat
                else:
                    first = self.capacity - self._write
                    self._buf[self._write :] = flat[:first]
                    self._buf[: n - first] = flat[first:]
                self._write = (self._write + n) % self.capacity
                self._filled = min(self.capacity, self._filled + n)
        return rms

    def snapshot(self) -> np.ndarray:
        with self._lock:
            if self._filled < self.capacity:
                return self._buf[: self._filled].copy()
            return np.concatenate(
                (self._buf[self._write :], self._buf[: self._write])
            )

    def recent_rms(self, window_sec: float = 0.25) -> float:
        snap = self.snapshot()
        if snap.size == 0:
            return 0.0
        w = min(snap.size, int(self.sample_rate * window_sec))
        tail = snap[-w:]
        return float(np.sqrt(np.mean(tail**2)))


class VideoVoidBuffer:
    """Deque of recent JPEG frames with motion metadata."""

    def __init__(self, max_seconds: float, max_fps: float = 4.0) -> None:
        self.max_len = max(4, int(max_seconds * max_fps))
        self._frames: Deque[VideoFrame] = deque(maxlen=self.max_len)
        self._lock = threading.Lock()

    def push(self, frame: VideoFrame) -> None:
        with self._lock:
            self._frames.append(frame)

    def latest(self) -> Optional[VideoFrame]:
        with self._lock:
            return self._frames[-1] if self._frames else None

    def snapshot(self) -> list[VideoFrame]:
        with self._lock:
            return list(self._frames)

    def max_motion(self) -> float:
        with self._lock:
            if not self._frames:
                return 0.0
            return max(f.motion_score for f in self._frames)


class SensoryVoid:
    """Combined void buffer for the intelligence daemon."""

    def __init__(self, sample_rate: int, buffer_seconds: float) -> None:
        self.audio = AudioVoidBuffer(sample_rate, buffer_seconds)
        self.video = VideoVoidBuffer(buffer_seconds)
        self._last_entropy = 0.0

    def entropy(self, audio_rms: float, motion: float, audio_weight: float = 0.45) -> float:
        # Normalized sensory change metric (0..1 approximate).
        a = min(1.0, audio_rms / 0.15)
        m = min(1.0, motion / 40.0)
        e = audio_weight * a + (1.0 - audio_weight) * m
        self._last_entropy = e
        return e

    def should_reason(self, threshold: float, audio_rms: float, motion: float) -> bool:
        return self.entropy(audio_rms, motion) >= threshold
