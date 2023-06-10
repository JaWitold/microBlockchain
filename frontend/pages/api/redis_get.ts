import Redis from 'ioredis'
import type { NextApiRequest, NextApiResponse } from 'next'

const redis = new Redis(process.env?.REDIS_URL || 'redis://redis:6379')
export default async function getHandler(req: NextApiRequest, res: NextApiResponse) {
    try {
        const key = req.query.key as string
        const value = await redis.get(key)
        res.status(200).json({ message: `Value in Redis: ${value}` })
    } catch (error) {
        console.error(error)
        res.status(500).json({ message: 'Error getting value from Redis' })
    }
}
