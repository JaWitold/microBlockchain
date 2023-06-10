import Redis from 'ioredis'
import type { NextApiRequest, NextApiResponse } from 'next'

const redis = new Redis(process.env?.REDIS_URL || 'redis://redis:6379')
export default async function pushHandler(req: NextApiRequest, res: NextApiResponse) {
    try {
        const value = req.query.value as string
        await redis.lpush('blockchain', value)
        res.status(200).json({ message: `Value pushed in Redis, ${value}` })
    } catch (error) {
        console.error(error)
        res.status(500).json({ message: 'Error setting value in Redis' })
    }
}
