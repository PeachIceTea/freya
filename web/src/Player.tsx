import classNames from "classnames"
import { useEffect, useRef, useState } from "react"
import { Image } from "react-bootstrap"
import { MdMenu, MdMenuOpen } from "react-icons/md"
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
import { formatDuration, useIsMobile } from "./common"
import { useLocale } from "./locales"
import { useStore } from "./store"

const PlaybackSpeeds = [1, 1.25, 1.5, 1.75, 2] as const

export default function Player() {
	const t = useLocale()
	const isMobile = useIsMobile()

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

	// Show extra controls.
	const [_showExtraControls, setShowExtraControls] = useState(false)
	const showExtraControls = _showExtraControls || !isMobile

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
	return isActive ? (
		<div
			className={classNames(
				"text-white",
				"z-3",
				"user-select-none",
				"rounded-top-3",
				"d-flex",
				"align-items-center",
				{
					"flex-column": isMobile,
				},
			)}
			hidden={!isActive}
			style={{
				backgroundColor: "#141414",
			}}
		>
			<audio src={fileUrl} ref={audioRef}></audio>
			<div className="d-flex w-100">
				<div className="d-flex">
					<Link to={`/book/${selectedBook.id}`}>
						<Image
							className="cover m-2 rounded"
							src={bookCoverURL(selectedBook.id)}
						/>
					</Link>

					<div className="d-flex flex-column justify-content-center">
						<Link
							to={`/book/${selectedBook.id}`}
							className="link-underline link-underline-opacity-0 link-light"
						>
							{selectedBook.title}
						</Link>
						<span className="text-secondary">{selectedBook.author}</span>
					</div>
				</div>
			</div>
			<div className="d-flex flex-column justify-content-center mx-5 w-100 px-2">
				<div className="d-flex justify-content-between my-3">
					<span>
						{formatDuration(
							audioRef.current?.currentTime,
							audioRef.current?.duration,
						)}
					</span>
					<input
						type="range"
						min="0"
						max={audioRef.current?.duration || 0}
						value={audioRef.current?.currentTime || 0}
						step={0.1}
						className="progress-bar mt-2 mx-3"
						style={{
							background: `linear-gradient(to right, #fff ${progress}%, #3a3a3a 0%)`,
						}}
						onChange={event => {
							seek(parseFloat(event.target.value))
						}}
					/>
					<span>{formatDuration(audioRef.current?.duration)}</span>
				</div>
				<div className="d-flex justify-content-between mb-2">
					{/* Mirror menu button on the left to center the controls */}
					<div hidden={!isMobile}>
						<MdMenu className="player-control opacity-0" />
					</div>
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
					<div
						className={classNames(
							{ "d-flex": isMobile, "d-none": !isMobile },
							"justify-content-center",
							"align-items-center",
						)}
						role="button"
						onClick={() => {
							setShowExtraControls(!showExtraControls)
						}}
					>
						{showExtraControls ? (
							<MdMenuOpen className="player-control" />
						) : (
							<MdMenu className="player-control" />
						)}
					</div>
				</div>
			</div>
			<div
				className={classNames(
					"justify-content-around",
					"d-flex",
					"align-items-center",
					"w-100",
					{
						"flex-column": !isMobile,
						"overflow-hidden": isMobile,
						"flex-shrink-1": !isMobile,
					},
				)}
				style={{
					// https://stackoverflow.com/a/52338132
					maxHeight: isMobile ? (showExtraControls ? 90 : 0) : "auto",
					transition: "all 1s ease-in-out",
				}}
			>
				<div className="d-flex flex-column w-50 mx-2 mb-2">
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
				<div className="d-flex flex-column w-50 mx-2 mb-2">
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
	) : null
}
