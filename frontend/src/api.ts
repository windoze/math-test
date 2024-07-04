const API_BASE = import.meta.env.VITE_API_BASE as string;

export interface Question {
    id: number;
    question: string;
    currentAnswer?: number;
}

export interface SubmitResponse {
    id: number;
    correct: boolean;
}

export interface Statistics {
    total: number,
    correct: number,
}

export async function newQuestion(): Promise<Question> {
    const body = (await fetch(`${API_BASE}/new-question`, {
        method: "POST",
    })).json();
    return body;
}

export async function submitAnswer(questionId: number, answer: number): Promise<SubmitResponse> {
    const body = (await fetch(`${API_BASE}/submit-answer`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            "id": questionId,
            "answer": answer,
        }),
    })).json();
    return body;
}

export async function todayScore(): Promise<Statistics> {
    const body = (await fetch(`${API_BASE}/today`)).json();
    return body;
}