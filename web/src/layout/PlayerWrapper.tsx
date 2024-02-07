import Player from "../Player"
import { useStore } from "../store"

export default function PlayerWrapper() {
	const { selectedBook, playing, volume, playbackSpeed } = useStore(
		({ selectedBook, playing, volume, playbackSpeed }) => ({
			selectedBook,
			playing,
			volume,
			playbackSpeed,
		}),
	)

	if (selectedBook === null) {
		return null
	}

	return (
		<Player
			playing={playing}
			selectedBook={selectedBook}
			volume={volume}
			playbackSpeed={playbackSpeed}
		/>
	)
}
