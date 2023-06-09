import Redis from 'ioredis'
import type { NextApiRequest, NextApiResponse } from 'next'

const redis = new Redis(process.env?.REDIS_URL || 'redis://redis:6379')
export default async function setHandler(req: NextApiRequest, res: NextApiResponse) {
    try {
        const key = req.query.key as string
        const value = req.query.value as string
        await redis.set(key, value)
        res.status(200).json({ message: `Value set in Redis, ${key}, ${value}` })
    } catch (error) {
        console.error(error)
        res.status(500).json({ message: 'Error setting value in Redis' })
    }
}
