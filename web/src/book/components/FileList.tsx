import { ListGroup } from "react-bootstrap"
import { TbPlayerPauseFilled, TbPlayerPlayFilled } from "react-icons/tb"

import { BookDetails, Chapter, File } from "../../api/books"
import { formatDuration } from "../../common"

export default function FileList({
	book,
	currentlyPlaying,
	onClick,
}: {
	book: BookDetails
	currentlyPlaying: boolean
	onClick: (file: File | Chapter) => void
}) {
	const list = book.chapters ?? book.files

	// Figure out progress for a item in the list.
	const playingFileIndex = book.files.findIndex(
		file => file.id === book.library?.fileId,
	)
	function getProgress(item: File | Chapter, position: number) {
		if (!book.library) {
			return 0
		}

		if ("duration" in item) {
			return item.id === book.library?.fileId
				? book.library?.progress / book.duration
				: position < playingFileIndex
					? 1
					: 0
		} else {
			return item.end < book.library.progress
				? 1
				: item.start < book.library.progress
					? (book.library.progress - item.start) / (item.end - item.start)
					: 0
		}
	}

	return (
		<>
			{list.map((item, position) => {
				const duration =
					"duration" in item ? item.duration : item.end - item.start
				const progress = getProgress(item, position)

				return (
					<ListGroup.Item key={item.id} className="p-0">
						<div className="mx-3 my-2 me-auto d-flex justify-content-between align-items-center">
							<div className="flex-grow-1" style={{ minWidth: 0 }}>
								<div className="fw-bold text-break">{item.name}</div>
								<div className="text-secondary">
									Duration: {formatDuration(duration)}
								</div>
							</div>
							<div
								className="details-control me-2"
								role="button"
								onClick={() => onClick(item)}
							>
								{currentlyPlaying && progress !== 0 && progress !== 1 ? (
									<TbPlayerPauseFilled />
								) : (
									<TbPlayerPlayFilled />
								)}
							</div>
						</div>
						<div
							className="bg-primary"
							style={{
								height: "0.25em",
								width: `${progress * 100}%`,
							}}
						></div>
					</ListGroup.Item>
				)
			})}
		</>
	)
}
