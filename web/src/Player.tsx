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
import { mutate } from "swr"
import { Link } from "wouter"

import Select from "./Select"
import { BookDetails, bookCoverURL } from "./api/books"
import { addBookToLibrary, updateProgress } from "./api/library"
import { formatDuration, useIsMobile } from "./common"
import { useLocale } from "./locales"
import { useStore } from "./store"

const PlaybackSpeeds = [1, 1.25, 1.5, 1.75, 2] as const

// We get the state of the store passed in to ensure that selectedBook and selectedBook.library
// are not null. Otherwise we would have to check for null all over the place since React doesn't
// allow us to return early from a component because of hooks.
function PlayerComponent({
	playing,
	selectedBook,
	volume,
	playbackSpeed,
	forceSeek,
}: {
	playing: boolean
	selectedBook: Required<BookDetails>
	volume: number
	playbackSpeed: number
	forceSeek: boolean
}) {
	const t = useLocale()
	const isMobile = useIsMobile()

	// Get data from library.
	const library = selectedBook.library
	const file = selectedBook.files.find(file => file.id === library.fileId)!
	const fileUrl = `/api/fs/audio/${file.id}`

	// In theory it might be cleaner to get the store functions via props as well, but that seems
	// like a hassle. I am not sure if this will cause React to rerender the Player component
	// multiple times on a single state change, but I don't think it will. 🙏
	const storeFn = useStore(state => {
		return {
			play: state.play,
			pause: state.pause,
			nextFile: state.nextFile,
			prevFile: state.prevFile,
			updateProgress: state.updateProgress,
			setVolume: state.setVolume,
			setPlaybackSpeed: state.setPlaybackSpeed,
			seekComplete: state.seekComplete,
		}
	})

	// Create a reference to the audio element.
	const audioRef = useRef<HTMLAudioElement>(null)

	// Check if we are using file or chapter mode.
	const chapterMode = selectedBook.chapters.length > 0
	const chapter =
		audioRef.current && selectedBook.chapters.length > 0
			? selectedBook.chapters.find(
					chapter =>
						audioRef.current!.currentTime > chapter.start &&
						audioRef.current!.currentTime < chapter.end,
				)
			: undefined
	const audioDuration = chapterMode
		? chapter
			? chapter.end - chapter.start
			: 0
		: file.duration
	const audioCurrentTime = chapterMode
		? chapter
			? audioRef.current!.currentTime - chapter.start
			: 0
		: audioRef.current?.currentTime ?? 0

	// Set audioRef to progress whenever a new book is selected.
	useEffect(() => {
		if (audioRef.current) {
			audioRef.current.currentTime = library.progress
		}

		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [audioRef, selectedBook.id])

	if (forceSeek && audioRef.current) {
		audioRef.current.currentTime = library.progress
		storeFn.seekComplete()
	}

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

	function skipBackwards() {
		if (chapter) {
			const previousChapter =
				selectedBook.chapters[selectedBook.chapters.indexOf(chapter) - 1]
			seek(previousChapter.start, true)
		} else {
			storeFn.prevFile()
		}
	}

	function skipForwards() {
		if (chapter) {
			seek(chapter.end + 1, true)
		} else {
			storeFn.nextFile()
		}
	}

	function seek(to: number, absolute = false) {
		if (audioRef.current) {
			audioRef.current.currentTime =
				chapter && !absolute ? to + chapter.start : to
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
	}, [audioRef])

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
			// Sync events with playing store.
			const handlePlay = () => {
				storeFn.play()
			}
			const handlePause = () => {
				storeFn.pause()
			}
			ref.addEventListener("play", handlePlay)
			ref.addEventListener("pause", handlePause)

			// Listen for end of audio to automatically play next file.
			const handleEnded = () => {
				if (!storeFn.nextFile()) {
					addBookToLibrary(selectedBook.id, "finished")
					mutate(`/book/${selectedBook.id}`)
				}
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
					storeFn.updateProgress(ref.currentTime)
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
	}, [audioRef, file, storeFn, selectedBook.id])

	// Update progress on the server every 30 seconds.
	useEffect(() => {
		const interval = setInterval(() => {
			updateProgress(
				selectedBook.id,
				file.id,
				audioRef.current?.currentTime ?? 0,
			)
		}, 30 * 1000)
		return () => clearInterval(interval)
	}, [audioRef, file.id, selectedBook.id])

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
			audioRef.current.playbackRate = playbackSpeed
		}
	}, [playbackSpeed, audioRef, file])
	const [isPlaybackSpeedOpen, setIsPlaybackSpeedOpen] = useState(false)

	// Calculate progress.
	const progress = (audioCurrentTime / audioDuration) * 100

	return (
		<div
			className={classNames(
				"text-white",
				"user-select-none",
				"rounded-top-3",
				"d-flex",
				"align-items-center",
				{
					"flex-column": isMobile,
				},
			)}
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
					<span>{formatDuration(audioCurrentTime, audioDuration)}</span>
					<input
						type="range"
						min="0"
						max={audioDuration || 0}
						value={audioCurrentTime || 0}
						step={0.1}
						className="progress-bar mt-2 mx-3"
						style={{
							background: `linear-gradient(to right, #fff ${progress}%, #3a3a3a 0%)`,
						}}
						onChange={event => {
							seek(parseFloat(event.target.value))
						}}
					/>
					<span>{formatDuration(audioDuration)}</span>
				</div>
				<div className="d-flex justify-content-between mb-2">
					{/* Mirror menu button on the left to center the controls */}
					<div hidden={!isMobile}>
						<MdMenu className="player-control opacity-0" />
					</div>
					<TbPlayerSkipBackFilled
						className={classNames({
							"player-control": true,
							"d-none": selectedBook.files.length === 1 && !chapterMode,
						})}
						role="button"
						onClick={skipBackwards}
					/>
					<TbRewindBackward30
						className="player-control"
						role="button"
						onClick={rewind30Seconds}
					/>
					{playing ? (
						<TbPlayerPauseFilled
							className="player-control"
							role="button"
							onClick={() => {
								storeFn.pause()
							}}
						/>
					) : (
						<TbPlayerPlayFilled
							className="player-control"
							role="button"
							onClick={() => {
								storeFn.play()
							}}
						/>
					)}
					<TbRewindForward30
						className="player-control"
						role="button"
						onClick={forward30Seconds}
					/>
					<TbPlayerSkipForwardFilled
						className={classNames({
							"player-control": true,
							"d-none": selectedBook.files.length === 1 && !chapterMode,
						})}
						role="button"
						onClick={skipForwards}
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
						"overflow-hidden": isMobile && !isPlaybackSpeedOpen,
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
							storeFn.setVolume(parseFloat(event.target.value))
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
						value={playbackSpeed}
						onChange={value => {
							storeFn.setPlaybackSpeed(value)
						}}
						onOpenChange={setIsPlaybackSpeedOpen}
						data-bs-theme="dark"
					/>
				</div>
			</div>
		</div>
	)
}

export default function Player() {
	const { selectedBook, playing, volume, playbackSpeed, forceSeek } = useStore(
		({ selectedBook, playing, volume, playbackSpeed, forceSeek }) => ({
			selectedBook,
			playing,
			volume,
			playbackSpeed,
			forceSeek,
		}),
	)

	if (selectedBook === null) {
		return null
	}

	return (
		<PlayerComponent
			playing={playing}
			selectedBook={selectedBook}
			volume={volume}
			playbackSpeed={playbackSpeed}
			forceSeek={forceSeek}
		/>
	)
}
