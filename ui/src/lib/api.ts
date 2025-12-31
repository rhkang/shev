const BASE_URL = '/api';

export interface Status {
	consumer_running: boolean;
	total_jobs: number;
	pending_jobs: number;
	running_jobs: number;
	completed_jobs: number;
	failed_jobs: number;
}

export interface Event {
	id: string;
	event_type: string;
	context: string;
	timestamp: string;
}

export interface Job {
	id: string;
	event: Event;
	handler_id: string;
	status: 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled';
	output: string | null;
	error: string | null;
	started_at: string | null;
	finished_at: string | null;
}

export interface EventHandler {
	id: string;
	event_type: string;
	shell: 'Pwsh' | 'Bash' | 'Sh';
	command: string;
	timeout: number | null;
}

export interface Timer {
	id: string;
	event_type: string;
	context: string;
	interval_secs: number;
}

export interface Schedule {
	id: string;
	event_type: string;
	context: string;
	scheduled_time: string;
	periodic: boolean;
}

export interface ReloadResult {
	success: boolean;
	handlers_loaded: number;
	timers_loaded: number;
	schedules_loaded: number;
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
	const res = await fetch(`${BASE_URL}${path}`, {
		headers: {
			'Content-Type': 'application/json',
			...options?.headers
		},
		...options
	});
	if (!res.ok) {
		const text = await res.text();
		throw new Error(text || res.statusText);
	}
	return res.json();
}

export const api = {
	// Status
	getStatus: () => request<Status>('/status'),

	// Jobs
	getJobs: () => request<Job[]>('/jobs'),
	getCompletedJobs: () => request<Job[]>('/jobs/completed'),
	getJob: (id: string) => request<Job>(`/jobs/${id}`),
	cancelJob: (id: string) => request<{ message: string }>(`/jobs/${id}/cancel`, { method: 'POST' }),

	// Events
	createEvent: (event_type: string, context: string) =>
		request<Event & { message: string }>('/events', {
			method: 'POST',
			body: JSON.stringify({ event_type, context })
		}),

	// Handlers
	getHandlers: () => request<EventHandler[]>('/handlers'),

	// Timers
	getTimers: () => request<Timer[]>('/timers'),

	// Schedules
	getSchedules: () => request<Schedule[]>('/schedules'),

	// Consumer
	startConsumer: () => request<{ message: string }>('/consumer/start', { method: 'POST' }),
	stopConsumer: () => request<{ message: string }>('/consumer/stop', { method: 'POST' }),

	// Reload
	reload: () => request<ReloadResult>('/reload', { method: 'POST' })
};
