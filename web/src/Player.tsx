import { useEffect, useRef, useState } from "react"
import { Form, Image } from "react-bootstrap"
import {
	TbPlayerPauseFilled,
	TbPlayerPlayFilled,
	TbPlayerSkipBackFilled,
	TbPlayerSkipForwardFilled,
	TbRewindBackward30,
	TbRewindForward30,
} from "react-icons/tb"
import { Link } from "wouter"

import Select from "./Select"
import { bookCoverURL } from "./api/books"
import { formatDuration } from "./common"
import { useLocale } from "./locales"
import { useStore } from "./store"

const PlaybackSpeeds = [1, 1.25, 1.5, 1.75, 2] as const

export default function Player() {
	const t = useLocale()

	// Get the current state from the store
	const state = useStore()
	const { playing, selectedBook, selectedFileIndex, volume } = state
	const file =
		selectedFileIndex !== null
			? selectedBook?.files[selectedFileIndex]
			: undefined
	const fileUrl = file ? `/api/fs/audio/${file.id}` : undefined
	const [progress, setProgress] = useState(0)

	// Reset progress when the file changes.
	useEffect(() => {
		setProgress(0)
	}, [file])

	// debugging log
	/* console.log({
		playing,
		selectedBook,
		selectedFileIndex,
		file,
		fileUrl,
		progress,
	}) */

	// Create a reference to the audio element
	const audioRef = useRef<HTMLAudioElement>(null)

	// Audio control functions.
	async function play() {
		if (audioRef.current) {
			await audioRef.current.play()
		}
	}
	async function pause() {
		if (audioRef.current) {
			await audioRef.current.pause()
		}
	}
	async function togglePlay() {
		if (audioRef.current) {
			if (audioRef.current.paused) {
				await audioRef.current.play()
			} else {
				await audioRef.current.pause()
			}
		}
	}
	function rewind30Seconds() {
		if (audioRef.current) {
			audioRef.current.currentTime -= 30
		}
	}
	function forward30Seconds() {
		if (audioRef.current) {
			audioRef.current.currentTime += 30
		}
	}
	function seek(to: number) {
		if (audioRef.current) {
			audioRef.current.currentTime = to
		}
	}

	// Keyboard shortcuts.
	useEffect(() => {
		async function handleKeyDown(event: KeyboardEvent) {
			// Ignore keydown events when the user is typing in an input.
			if (event.target instanceof HTMLInputElement) {
				return
			}

			switch (event.key) {
				case " ":
					event.preventDefault()
					await togglePlay()
					break
				case "ArrowLeft":
					event.preventDefault()
					rewind30Seconds()
					break
				case "ArrowRight":
					event.preventDefault()
					forward30Seconds()
					break
				default:
					break
			}
		}
		document.addEventListener("keydown", handleKeyDown)
		return () => {
			document.removeEventListener("keydown", handleKeyDown)
		}
	}, [state])

	// Sync playing state with the audio element.
	useEffect(() => {
		async function syncPlayingState() {
			if (audioRef.current) {
				if (playing) {
					await play()
				} else {
					await pause()
				}
			}
		}
		syncPlayingState()
	}, [playing, audioRef, file])

	// Register event listeners for the audio element.
	useEffect(() => {
		if (audioRef.current) {
			const ref = audioRef.current
			// Sync events with playing state.
			const handlePlay = () => {
				state.play()
			}
			const handlePause = () => {
				state.pause()
			}
			ref.addEventListener("play", handlePlay)
			ref.addEventListener("pause", handlePause)

			// Listen for end of audio to automatically play next file.
			const handleEnded = () => {
				state.nextFile()
			}
			ref.addEventListener("ended", handleEnded)

			// Update progress bar.
			const handleTimeUpdate = () => {
				if (
					ref.duration !== undefined &&
					!Number.isNaN(ref.duration) &&
					ref.duration > 0 &&
					ref.currentTime !== undefined &&
					!Number.isNaN(ref.currentTime)
				) {
					setProgress((ref.currentTime / ref.duration) * 100)
				}
			}
			ref.addEventListener("timeupdate", handleTimeUpdate)

			// Clean up event listeners.
			return () => {
				ref.removeEventListener("play", handlePlay)
				ref.removeEventListener("pause", handlePause)
				ref.removeEventListener("ended", handleEnded)
				ref.removeEventListener("timeupdate", handleTimeUpdate)
			}
		}
	}, [audioRef, state])

	// Control volume.
	useEffect(() => {
		if (audioRef.current) {
			audioRef.current.volume = Math.min(volume, 1)
		}
	}, [volume, audioRef])

	// Control playback speed.
	useEffect(() => {
		if (audioRef.current) {
			audioRef.current.playbackRate = state.playbackSpeed
		}
	}, [state.playbackSpeed, audioRef, file])

	// If there is no selected book or file, don't render anything.
	const isActive =
		selectedBook && selectedFileIndex !== null && file !== undefined
	return (
		<>
			{isActive && (
				<div
					className="fixed-bottom grid text-white position-fixed bottom-0 z-3 justify-content-between user-select-none"
					hidden={!isActive}
					style={{
						backgroundColor: "#141414",
					}}
				>
					<div className="g-col-4 d-flex">
						<Link to={`/book/${selectedBook.id}`}>
							<Image
								className="cover m-2 rounded"
								src={bookCoverURL(selectedBook.id)}
							/>
						</Link>

						<div className="d-flex flex-column justify-content-center fs-5">
							<Link
								to={`/book/${selectedBook.id}`}
								className="link-underline link-underline-opacity-0 link-light"
							>
								{selectedBook.title}
							</Link>
							<span className="text-secondary">{selectedBook.author}</span>
						</div>
					</div>
					<div className="g-col-4 d-flex flex-column justify-content-center ">
						<div className="d-flex justify-content-between mb-2">
							<TbPlayerSkipBackFilled
								className="player-control"
								role="button"
								onClick={() => {
									state.prevFile()
								}}
							/>
							<TbRewindBackward30
								className="player-control"
								role="button"
								onClick={() => {
									if (audioRef.current) {
										audioRef.current.currentTime -= 30
									}
								}}
							/>
							{playing ? (
								<TbPlayerPauseFilled
									className="player-control"
									role="button"
									onClick={() => {
										state.pause()
									}}
								/>
							) : (
								<TbPlayerPlayFilled
									className="player-control"
									role="button"
									onClick={() => {
										state.play()
									}}
								/>
							)}
							<TbRewindForward30
								className="player-control"
								role="button"
								onClick={() => {
									if (audioRef.current) {
										audioRef.current.currentTime += 30
									}
								}}
							/>
							<TbPlayerSkipForwardFilled
								className="player-control"
								role="button"
								onClick={() => {
									state.nextFile()
								}}
							/>
						</div>
						<input
							type="range"
							min="0"
							max={audioRef.current?.duration || 0}
							value={audioRef.current?.currentTime || 0}
							step={0.1}
							className="progress-bar mb-2"
							style={{
								background: `linear-gradient(to right, #fff ${progress}%, #3a3a3a 0%)`,
							}}
							onChange={event => {
								seek(parseFloat(event.target.value))
							}}
						/>
						<div className="d-flex justify-content-between">
							<span>
								{formatDuration(
									audioRef.current?.currentTime,
									audioRef.current?.duration,
								)}
							</span>

							<span>{formatDuration(audioRef.current?.duration)}</span>
						</div>
					</div>
					<div className="g-col-4 d-flex justify-content-around align-items-center flex-column">
						<div className="d-flex flex-column w-50">
							<span className="mb-1">{t("player--volume")}</span>
							<input
								type="range"
								min="0"
								max="1"
								step="0.001"
								value={volume}
								style={{
									background: `linear-gradient(to right, #fff ${volume * 100}%, #3a3a3a 0%)`,
								}}
								onChange={event => {
									state.setVolume(parseFloat(event.target.value))
								}}
							/>
						</div>
						<div className="d-flex flex-column w-50">
							<span className="mb-1">{t("player--playback-speed")}</span>
							<Select
								options={PlaybackSpeeds.map(speed => ({
									value: speed,
									label: `${speed}x`,
								}))}
								value={state.playbackSpeed}
								onChange={value => {
									state.setPlaybackSpeed(value)
								}}
								data-bs-theme="dark"
							/>
						</div>
					</div>
				</div>
			)}
			<audio src={fileUrl} ref={audioRef}></audio>
		</>
	)
}
