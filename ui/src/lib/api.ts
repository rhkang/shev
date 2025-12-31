const BASE_URL = '/api';

export interface Status {
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
	status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
	output: string | null;
	error: string | null;
	started_at: string | null;
	finished_at: string | null;
}

export interface EventHandler {
	id: string;
	event_type: string;
	shell: string;
	command: string;
	timeout: number | null;
	env: Record<string, string>;
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

export type JobStatus = Job['status'];

export const JOB_STATUSES: JobStatus[] = ['pending', 'running', 'completed', 'failed', 'cancelled'];

export const api = {
	// Status
	getStatus: () => request<Status>('/status'),

	// Jobs
	getJobs: () => request<Job[]>('/jobs'),
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
	createHandler: (data: { event_type: string; shell: string; command: string; timeout?: number; env?: Record<string, string> }) =>
		request<EventHandler>('/handlers', {
			method: 'POST',
			body: JSON.stringify(data)
		}),
	updateHandler: (event_type: string, data: { shell?: string; command?: string; timeout?: number | null; env?: Record<string, string> }) =>
		request<EventHandler>(`/handlers/${encodeURIComponent(event_type)}`, {
			method: 'PUT',
			body: JSON.stringify(data)
		}),
	deleteHandler: (event_type: string) =>
		request<{ deleted: boolean }>(`/handlers/${encodeURIComponent(event_type)}`, { method: 'DELETE' }),

	// Timers
	getTimers: () => request<Timer[]>('/timers'),
	createTimer: (data: { event_type: string; interval_secs: number; context?: string }) =>
		request<Timer>('/timers', {
			method: 'POST',
			body: JSON.stringify(data)
		}),
	updateTimer: (event_type: string, data: { interval_secs?: number; context?: string }) =>
		request<Timer>(`/timers/${encodeURIComponent(event_type)}`, {
			method: 'PUT',
			body: JSON.stringify(data)
		}),
	deleteTimer: (event_type: string) =>
		request<{ deleted: boolean }>(`/timers/${encodeURIComponent(event_type)}`, { method: 'DELETE' }),

	// Schedules
	getSchedules: () => request<Schedule[]>('/schedules'),
	createSchedule: (data: { event_type: string; scheduled_time: string; context?: string; periodic?: boolean }) =>
		request<Schedule>('/schedules', {
			method: 'POST',
			body: JSON.stringify(data)
		}),
	updateSchedule: (event_type: string, data: { scheduled_time?: string; context?: string; periodic?: boolean }) =>
		request<Schedule>(`/schedules/${encodeURIComponent(event_type)}`, {
			method: 'PUT',
			body: JSON.stringify(data)
		}),
	deleteSchedule: (event_type: string) =>
		request<{ deleted: boolean }>(`/schedules/${encodeURIComponent(event_type)}`, { method: 'DELETE' }),

	// Reload
	reload: () => request<ReloadResult>('/reload', { method: 'POST' })
};
