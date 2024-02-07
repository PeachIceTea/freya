import { useEffect, useState } from "react"
import {
	Alert,
	Button,
	Container,
	Form,
	Image,
	Tab,
	Tabs,
} from "react-bootstrap"
import { useLocation } from "wouter"

import type { Error } from "../api/api"
import { uploadBook } from "../api/books"
import { Entry, FileInfo, getFileInfo, getTmpCoverImageURL } from "../api/fs"
import { useTitle } from "../common"
import { useLocale } from "../locales"
import { FileSelect } from "./components/FileSelect"

type CoverTabs = "upload" | "select" | "extracted" | "url"

export default function NewBook() {
	useTitle("new-book--title")
	const t = useLocale()
	const [_location, setLocation] = useLocation()

	const [error, setError] = useState<Error | null>(null)
	const [loading, setLoading] = useState(false)

	// Info received from ffprobe.
	const [fileInfo, setFileInfo] = useState<FileInfo | null>()
	async function fetchFileInfo(path: string) {
		const res = await getFileInfo(path)
		if (res.success) {
			setFileInfo(res.data.info)

			// Move to the extracted cover tab if we have a cover.
			if (res.data.info.cover) {
				setActiveCoverTab("extracted")
			}
		}
	}

	// Form data.
	const [files, _setFiles] = useState<Entry[]>([])
	function setFiles(files: Entry[]) {
		_setFiles(files)

		// If we have a file, fetch audio book info.
		if (files.length > 0) {
			fetchFileInfo(files[0].path)
		}
	}
	const [title, setTitle] = useState("")
	const [author, setAuthor] = useState("")
	const [activeCoverTab, setActiveCoverTab] = useState<CoverTabs>("select")

	// Cover data.
	const [selectCover, setSelectCover] = useState<string | null>(null)
	const [coverUpload, setCoverUpload] = useState<File | null>(null)
	const [coverUrl, setCoverUrl] = useState<string>("")
	const [coverUrlValid, setCoverUrlValid] = useState(false)
	useEffect(() => {
		if (coverUrl) {
			setCoverUrlValid(true)
		}
	}, [coverUrl, setCoverUrlValid])

	async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
		event.preventDefault()
		if (loading) {
			return
		}
		setLoading(true)

		// Get data from form.
		const form = event.currentTarget
		const formData = new FormData(form)

		// Append the cover to form data.
		let cover
		switch (activeCoverTab) {
			case "select":
				cover = selectCover
				break
			case "upload":
				cover = coverUpload
				break
			case "url":
				cover = coverUrl
				break
			case "extracted":
				cover = fileInfo?.cover
				break
		}
		if (cover) {
			formData.append("cover", cover)
		}

		// Append files to form data.
		files.forEach(file => {
			formData.append("files", file.path)
		})

		// Send form data to server.
		const res = await uploadBook(formData)
		setLoading(false)

		if (!res.success) {
			return setError(res)
		}

		// Redirect to new book page.
		setLocation(`/book/${res.data.bookId}`)
	}

	return (
		<Container>
			<h1>{t("new-book--title")}</h1>
			{error && (
				<Alert variant="danger">
					{t(
						error.errorCode,
						error.value
							? {
									error: error.value,
								}
							: undefined,
					)}
				</Alert>
			)}

			<Form onSubmit={handleSubmit}>
				<div className="mb-3">
					<Form.Label htmlFor="files-input">
						{t("new-book--files-label")}
					</Form.Label>
					<FileSelect
						id="files-input"
						fileCategory="audio"
						multiSelect={true}
						onChange={setFiles}
					/>
				</div>
				<div className="mb-3">
					<Form.Label htmlFor="title-input">
						{t("new-book--title-label")}
					</Form.Label>
					<Form.Control
						id="title-input"
						name="title"
						type="text"
						value={title}
						onChange={event => setTitle(event.target.value)}
						required
					/>
					{fileInfo?.title && (
						<Button
							type="button"
							variant="link"
							onClick={() => setTitle(fileInfo.title!)}
						>
							{t("new-book--title-suggest", { title: fileInfo.title })}
						</Button>
					)}
				</div>
				<div className="mb-3">
					<Form.Label htmlFor="author-input">
						{t("new-book--author-label")}
					</Form.Label>
					<Form.Control
						id="author-input"
						name="author"
						type="text"
						value={author}
						onChange={event => setAuthor(event.target.value)}
						required
					/>
					{fileInfo?.author && (
						<Button
							type="button"
							variant="link"
							onClick={() => setAuthor(fileInfo.author!)}
						>
							{t("new-book--author-suggest", { author: fileInfo.author })}
						</Button>
					)}
				</div>
				<div className="mb-3">
					<Form.Label htmlFor="cover-input">
						{t("new-book--cover-label")}
					</Form.Label>
					<Tabs
						activeKey={activeCoverTab}
						onSelect={activeKey => setActiveCoverTab(activeKey as CoverTabs)}
						className="mb-3"
						variant="underline"
						fill
					>
						{/* Select cover from server. */}
						<Tab eventKey="select" title={t("new-book--tab-cover-select")}>
							<Alert variant="info">
								{t("new-book--cover-select-explainer")}{" "}
								{t("new-book--cover-generic-explainer")}
							</Alert>
							<FileSelect
								id="cover-input"
								fileCategory="image"
								multiSelect={false}
								onChange={files => {
									setSelectCover(files[0].path ?? null)
								}}
							/>
							<div className="d-flex justify-content-center align-items-center mt-2">
								{selectCover && (
									<Image
										src={getTmpCoverImageURL(selectCover)}
										thumbnail={true}
									/>
								)}
							</div>
						</Tab>

						{/* Upload cover. */}
						<Tab eventKey="upload" title={t("new-book--tab-cover-upload")}>
							<Alert variant="info">
								{t("new-book--cover-upload-explainer")}{" "}
								{t("new-book--cover-generic-explainer")}
							</Alert>
							<Form.Control
								type="file"
								accept="image/*"
								onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
									const file = event.target.files?.[0]
									if (file) {
										setCoverUpload(file)
									} else {
										setCoverUpload(null)
									}
								}}
							/>
							<div className="d-flex justify-content-center align-items-center mt-2">
								{coverUpload && (
									<Image
										src={URL.createObjectURL(coverUpload)}
										thumbnail={true}
									/>
								)}
							</div>
						</Tab>

						{/* Donwload cover from URL. */}
						<Tab eventKey="url" title={t("new-book--tab-cover-url")}>
							<Alert variant="info">
								{t("new-book--cover-url-explainer")}{" "}
								{t("new-book--cover-generic-explainer")}
							</Alert>
							<Form.Control
								type="url"
								placeholder={t("new-book--cover-url-placeholder")}
								value={coverUrl}
								onChange={event => setCoverUrl(event.target.value)}
							/>
							<div className="d-flex justify-content-center align-items-center mt-2">
								{coverUrl && coverUrlValid && (
									<Image
										src={coverUrl}
										thumbnail={true}
										onError={() => setCoverUrlValid(false)}
									/>
								)}
							</div>
						</Tab>

						{/* Cover extracted from audio file. */}
						<Tab
							eventKey="extracted"
							title={t("new-book--tab-cover-extracted")}
							disabled={!fileInfo?.cover}
						>
							<Alert variant="info">
								{t("new-book--cover-extracted-explainer")}{" "}
								{t("new-book--cover-generic-explainer")}
							</Alert>
							<div className="d-flex justify-content-center align-items-center">
								<Image
									src={getTmpCoverImageURL(fileInfo?.cover)}
									thumbnail={true}
								/>
							</div>
						</Tab>
					</Tabs>
				</div>

				<Button type="submit" variant="primary" disabled={loading}>
					{t("new-book--submit-button")}
				</Button>
			</Form>
		</Container>
	)
}
