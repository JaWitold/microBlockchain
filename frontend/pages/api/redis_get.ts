import Redis from "ioredis";
import type { NextApiRequest, NextApiResponse } from "next";

// Create a Redis client
const redis = new Redis(process.env?.REDIS_URL || "redis://redis:6379");

// Define the get endpoint
export default async function getHandler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  try {
    // Get a value from Redis
    const value = await redis.get("myKey");

    res.status(200).json({ message: `Value in Redis: ${value}` });
  } catch (error) {
    console.error(error);
    res.status(500).json({ message: "Error getting value from Redis" });
  }
}
