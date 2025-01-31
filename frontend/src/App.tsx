import React, { useEffect, useState } from 'react'
import './App.css'
import { Question, Statistics, newQuestion, submitAnswer, todayScore } from './api'

function QuestionDisplay(question: Question) {
    return (
        <div style={{ fontSize: "2.25rem" }}
            className='text-2xl font-mono font-black text-left bg-green-300 border-0 py-4 px-3 rounded text-base mt-10'>
            {question.question} = {question.answer}
        </div>
    );
}

function ScoreDisplay(stat: Statistics) {
    let accuracy = 0;
    if (stat.total > 0) {
        accuracy = stat.correct / stat.total;
    }
    let color = "text-red-800";
    if (accuracy >= 0.9) {
        color = "text-green-800";
    } else if (accuracy >= 0.8) {
        color = "text-yellow-800";
    }
    return (
        <div className='mt-5 text-3xl text-center grid gap-4 grid-cols-2'>
            <div>
                做对：
            </div>
            <div>
                {stat.correct}
            </div>
            <div>
                总数：
            </div>
            <div>
                {stat.total}
            </div>
            <div>
                正确率：
            </div>
            <div className={color}>
                {Math.round(accuracy * 100)}%
            </div>
        </div>
    );
}

function NumberButton(digit: number, question: Question, setQuestion: React.Dispatch<React.SetStateAction<Question>>) {
    return (
        <button
            className='text-4xl text-center text-white bg-green-800 border-0 py-4 px-3 focus:outline-none hover:bg-green-700 rounded mt-4 md:mt-4'
            onClick={() => {
                setQuestion({
                    id: question.id,
                    question: question.question,
                    answer: (question.answer ?? 0) * 10 + digit
                });
            }}>{digit}</button>
    );
}

function Backspace(question: Question, setQuestion: React.Dispatch<React.SetStateAction<Question>>) {
    let newAnswer: number | undefined = undefined;
    if ((question.answer === undefined) || (question.answer < 10)) {
        newAnswer = undefined;
    } else {
        newAnswer = Math.floor((question.answer) / 10);
    }
    return (
        <button style={{ fontSize: "1.875rem" }}
            className='flex justify-center items-center text-white bg-red-800 border-0 py-2 px-2 focus:outline-none hover:bg-red-700 rounded text-base mt-4 md:mt-4 col-span-2'
            onClick={() => {
                setQuestion({
                    id: question.id,
                    question: question.question,
                    answer: newAnswer,
                });
            }}>
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5}
                stroke="currentColor" className="size-9">
                <path strokeLinecap="round" strokeLinejoin="round"
                    d="M12 9.75 14.25 12m0 0 2.25 2.25M14.25 12l2.25-2.25M14.25 12 12 14.25m-2.58 4.92-6.374-6.375a1.125 1.125 0 0 1 0-1.59L9.42 4.83c.21-.211.497-.33.795-.33H19.5a2.25 2.25 0 0 1 2.25 2.25v10.5a2.25 2.25 0 0 1-2.25 2.25h-9.284c-.298 0-.585-.119-.795-.33Z" />
            </svg>
        </button>
    );
}

function SubmitButton(question: Question, setQuestion: React.Dispatch<React.SetStateAction<Question>>, setStat: React.Dispatch<React.SetStateAction<Statistics>>, setCorrect: React.Dispatch<React.SetStateAction<boolean | undefined>>) {
    return (
        <button style={{ fontSize: "1.875rem" }}
            className='text-2xl text-center text-white bg-blue-800 border-0 py-4 px-3 focus:outline-none hover:bg-blue-700 rounded text-base mt-4 md:mt-4 col-start-4 col-span-2'
            onClick={() => {
                if (question.answer !== undefined) {
                    console.log(`Submitting ${question.id} ${question.answer}`);
                    submitAnswer(question.id, question.answer!).then((result) => {
                        setCorrect(result.correct);
                        setTimeout(() => {
                            setCorrect(undefined);
                        }, 2000);
                        todayScore().then(setStat).catch(console.error);
                        newQuestion().then(setQuestion).catch(console.error);
                    }).catch(console.error);
                }
            }}>提交</button>
    );
}

function ResultOverlay(correct: boolean | undefined) {
    if (correct === undefined) {
        return (<></>);
    }
    if (correct) {
        return (<div className='col-start-1 row-start-1 z-10'>
            <img src="check-mark.svg" />
        </div>);
    } else {
        return (<div className='col-start-1 row-start-1 z-10'>
            <img src="wrong-mark.svg" />
        </div>);
    }
}

function App() {
    const [stat, setStat] = useState<Statistics>({
        total: 0,
        correct: 0,
    } as Statistics);
    const [question, setQuestion] = useState<Question>({
        question: "Loading...",
    } as Question);
    const [correct, setCorrect] = useState<boolean | undefined>(undefined);
    useEffect(() => {
        todayScore().then(setStat).catch(console.error);
    }, []);
    useEffect(() => {
        newQuestion().then(setQuestion).catch(console.error);
    }, []);

    return (
        <div className='grid w-full max-w-md m-1.5'>
            {ResultOverlay(correct)}
            <div className='col-start-1 row-start-1'>
                {QuestionDisplay(question)}
                <div className='grid gap-4 grid-cols-5 mt-10'>
                    {NumberButton(1, question, setQuestion)}
                    {NumberButton(2, question, setQuestion)}
                    {NumberButton(3, question, setQuestion)}
                    {NumberButton(4, question, setQuestion)}
                    {NumberButton(5, question, setQuestion)}
                </div>
                <div className='grid gap-4 grid-cols-5 mt-10'>
                    {NumberButton(6, question, setQuestion)}
                    {NumberButton(7, question, setQuestion)}
                    {NumberButton(8, question, setQuestion)}
                    {NumberButton(9, question, setQuestion)}
                    {NumberButton(0, question, setQuestion)}
                </div>
                <div className='grid gap-4 grid-cols-5 mt-10'>
                    {Backspace(question, setQuestion)}
                    {SubmitButton(question, setQuestion, setStat, setCorrect)}
                </div>
                {ScoreDisplay(stat)}
                <div className='mt-5 text-3xl text-center'>
                    <a href="/last7"
                        className='mr-5 text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800"'>最近7天</a>
                    <a href="/last30"
                        className='ml-5 mr-5 text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800"'>最近30天</a>
                    <a href="/mistakes"
                        className='ml-5 text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800"'>错题本</a>
                </div>
            </div>
        </div>
    )
}

export default App
