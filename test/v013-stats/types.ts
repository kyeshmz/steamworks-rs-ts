import { init } from '../..'

const { stats, achievement } = init(3104020)
const game: Promise<bigint> = stats.requestGlobalStats(7)
const integer: number | null = stats.getUserStatInt(123n, 'score')
const decimal: number | null = stats.getUserStatFloat(123n, 'speed')
const unlocked: boolean | null = stats.getUserAchievement(123n, 'win')
const image: { width: number; height: number; rgba: Buffer } | null = achievement.getIconRgba('win')
stats.requestUserStats(123n)
void [game, integer, decimal, unlocked, image]
