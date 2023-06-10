import Redis from 'ioredis'
import type { NextApiRequest, NextApiResponse } from 'next'

const redis = new Redis(process.env?.REDIS_URL || 'redis://redis:6379')
export default async function rangeHandler(req: NextApiRequest, res: NextApiResponse) {
    try {
        const range = req.query.range as string
        const blocks = await redis.lrange('blockchain', -parseInt(range), -1)
        res.status(200).json({ message: `Value in Redis`, blocks })
    } catch (error) {
        console.error(error)
        res.status(500).json({ message: 'Error getting values from Redis' })
    }
}
