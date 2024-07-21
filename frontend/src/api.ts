const API_BASE = import.meta.env.VITE_API_BASE as string;

export interface Question {
    id: number;
    question: string;
    answer?: number;
}

export interface SubmitResponse {
    id: number;
    correct: boolean;
}

export interface Statistics {
    total: number,
    correct: number,
}

export interface StatisticsWithDate {
    date: string,
    total: number,
    correct: number,
}

export interface MultiStatistics {
    scores: StatisticsWithDate[],
    overall: Statistics,
}

export async function newQuestion(): Promise<Question> {
    return (await fetch(`${API_BASE}/new-question`, {
        method: "POST",
    })).json();
}

export async function submitAnswer(questionId: number, answer: number): Promise<SubmitResponse> {
    return (await fetch(`${API_BASE}/submit-answer`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            "id": questionId,
            "answer": answer,
        }),
    })).json();
}

export async function todayScore(): Promise<Statistics> {
    return (await fetch(`${API_BASE}/today`)).json();
}

export async function lastNScore(n: number): Promise<MultiStatistics> {
    return (await fetch(`${API_BASE}/last${n}`)).json();
}

export async function mistakeCollection(): Promise<Question[]> {
    return (await fetch(`${API_BASE}/mistake-collection`)).json();
}
