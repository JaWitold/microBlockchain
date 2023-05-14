import Redis from "ioredis";
import type { NextApiRequest, NextApiResponse } from "next";

// Create a Redis client
const redis = new Redis(process.env?.REDIS_URL || "redis://redis:6379");

// Define the set endpoint
export default async function setHandler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  try {
    // Set a value in Redis
    await redis.set("myKey", "Hello Redis!");

    res.status(200).json({ message: "Value set in Redis" });
  } catch (error) {
    console.error(error);
    res.status(500).json({ message: "Error setting value in Redis" });
  }
}
