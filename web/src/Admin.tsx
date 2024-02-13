import { Button, Container } from "react-bootstrap"

// TODO: Fix this rush job.

export default function AdminPage() {
	return (
		<Container>
			<div>
				<h1>Admin</h1>
			</div>
			<Button variant="primary" onClick={rediscoverChapters}>
				Rediscover Chapters
			</Button>
		</Container>
	)
}

async function rediscoverChapters() {
	// Send a POST request to the API.
	await fetch("/api/admin/rediscover-chapters", {
		method: "POST",
	})
}
