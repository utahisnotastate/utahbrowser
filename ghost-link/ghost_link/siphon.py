"""Peripheral Siphon — non-blocking camera and microphone direct-stream ingestion."""

from __future__ import annotations

import logging
import threading
import time
from typing import Callable, Optional

import numpy as np

from .config import GhostConfig
from .void_buffer import AudioChunk, SensoryVoid, VideoFrame

logger = logging.getLogger("ghost_link.siphon")


class PeripheralSiphon:
    """
    Starts isolated capture threads for video and audio.
    Frame-skipping: 1 frame / frame_interval_ms unless motion exceeds threshold.
    """

    def __init__(self, config: GhostConfig, void: SensoryVoid) -> None:
        self.config = config
        self.void = void
        self._stop = threading.Event()
        self._threads: list[threading.Thread] = []
        self._on_audio: Optional[Callable[[float], None]] = None
        self._on_motion: Optional[Callable[[float], None]] = None

    def on_audio_energy(self, cb: Callable[[float], None]) -> None:
        self._on_audio = cb

    def on_motion(self, cb: Callable[[float], None]) -> None:
        self._on_motion = cb

    def start(self) -> None:
        if self.config.enable_audio:
            t = threading.Thread(target=self._audio_loop, name="ghost-audio", daemon=True)
            t.start()
            self._threads.append(t)
        if self.config.enable_camera:
            t = threading.Thread(target=self._video_loop, name="ghost-video", daemon=True)
            t.start()
            self._threads.append(t)
        logger.info("Peripheral Siphon active (audio=%s camera=%s)", self.config.enable_audio, self.config.enable_camera)

    def stop(self) -> None:
        self._stop.set()
        for t in self._threads:
            t.join(timeout=2.0)

    def _audio_loop(self) -> None:
        try:
            import sounddevice as sd
        except ImportError:
            logger.error("sounddevice not installed — audio siphon disabled")
            return

        block = int(self.config.sample_rate * 0.1)

        def callback(indata, _frames, _time, status) -> None:
            if status:
                logger.warning("audio status: %s", status)
            chunk = np.asarray(indata[:, 0], dtype=np.float32)
            rms = self.void.audio.push(chunk)
            if self._on_audio:
                self._on_audio(rms)

        try:
            with sd.InputStream(
                channels=1,
                samplerate=self.config.sample_rate,
                blocksize=block,
                dtype="float32",
                callback=callback,
            ):
                while not self._stop.is_set():
                    time.sleep(0.1)
        except Exception as e:
            logger.error("audio siphon failed: %s", e)

    def _video_loop(self) -> None:
        try:
            import cv2
        except ImportError:
            logger.error("opencv not installed — video siphon disabled")
            return

        cap = cv2.VideoCapture(self.config.camera_index)
        if not cap.isOpened():
            logger.error("camera %s not available", self.config.camera_index)
            return

        prev_gray = None
        last_push = 0.0
        interval = self.config.frame_interval_ms / 1000.0

        while not self._stop.is_set():
            ok, frame = cap.read()
            if not ok:
                time.sleep(0.2)
                continue

            gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
            motion = 0.0
            if prev_gray is not None:
                diff = cv2.absdiff(gray, prev_gray)
                motion = float(np.mean(diff))
            prev_gray = gray

            if self._on_motion and motion >= self.config.motion_threshold:
                self._on_motion(motion)

            now = time.time()
            fast_path = motion >= self.config.motion_threshold
            if fast_path or (now - last_push) >= interval:
                ok_enc, buf = cv2.imencode(
                    ".jpg", frame, [int(cv2.IMWRITE_JPEG_QUALITY), 72]
                )
                if ok_enc:
                    vf = VideoFrame(
                        jpeg_bytes=buf.tobytes(),
                        timestamp=now,
                        motion_score=motion,
                    )
                    self.void.video.push(vf)
                    last_push = now

            time.sleep(0.02)

        cap.release()
        logger.info("video siphon stopped")
