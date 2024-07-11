import {useEffect, useState} from "react";
import {lastNScore, MultiStatistics, StatisticsWithDate} from "./api.ts";

function Scores(ms: MultiStatistics) {
    return <div className='relative overflow-x-auto'>
        <table className='w-full max-w-md m-1.5 text-sm text-left rtl:text-right text-gray-500 dark:text-gray-400'>
            <thead className='text-xs text-gray-900 uppercase dark:text-gray-400'>
            <tr>
                <th scope="col" className="px-6 py-3">
                    日期
                </th>
                <th scope="col" className="px-6 py-3">
                    总数
                </th>
                <th scope="col" className="px-6 py-3">
                    正确
                </th>
            </tr>
            </thead>
            <tbody key={"tbody"}>
            {ms.scores.map((s: StatisticsWithDate, i: number) => {
                return <tr key={`tr-${i.toString()}`} className='bg-white dark:bg-gray-800'>
                    <th key={`td-${i}-1`} scope='row'
                        className='px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white'>
                        {new Date(s.date).toDateString()}
                    </th>
                    <td key={`td-${i}-2`} className={'px-6 py-4'}>
                        {s.total}
                    </td>
                    <td key={`td-${i}-3`} className={'px-6 py-4'}>
                        {s.correct}
                    </td>
                </tr>;
            })}
            </tbody>
        </table>
        <a href="/">
            <svg className="w-6 h-6 text-gray-800 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg"
                 width="24" height="24" fill="none" viewBox="0 0 24 24">
                <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M5 12h14M5 12l4-4m-4 4 4 4"/>
            </svg>
        </a>
    </div>
}

export function Last7Days() {
    const [stat, setStat] = useState<MultiStatistics>({
        scores: [],
        overall: {
            total: 0,
            correct: 0,
        }
    } as MultiStatistics);
    useEffect(() => {
        lastNScore(7).then(setStat).catch(console.error);
    }, []);
    return Scores(stat);
}

export function Last30Days() {
    const [stat, setStat] = useState<MultiStatistics>({
        scores: [],
        overall: {
            total: 0,
            correct: 0,
        }
    } as MultiStatistics);
    useEffect(() => {
        lastNScore(30).then(setStat).catch(console.error);
    }, []);
    return Scores(stat);
}
