import {useEffect, useState} from "react";
import {lastNScore, MultiStatistics, StatisticsWithDate} from "./api.ts";

function score(s: StatisticsWithDate) {
    const d = new Date(s.date);
    return (<ul>
            {d.toDateString()} {s.correct}/{s.total}
        </ul>
    )
}

function Scores(ms: MultiStatistics) {
    // TODO: Show bar chart
    return (<div>
            --- Last {ms.scores.length} Days ---
            <ul>
                {ms.scores.map((s) => score(s))}
            </ul>
        </div>
    )
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
