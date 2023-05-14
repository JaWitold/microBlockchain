import type { NextApiRequest, NextApiResponse } from 'next';
import amqp from 'amqplib/callback_api';

// http://localhost:3001/api/rabbit_put?value=123
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  try {
    const value = req.query.value as string;
    if (!value) {
      return res.status(400).json({ error: 'Missing value in query string' });
    }

    amqp.connect(process.env?.RABBITMQ_URL || "amqp://guest:guest@rabbitmq:5672", function (err, conn) {
      if (err) {
        console.error(err);
        return res.status(500).json({ error: 'Failed to connect to RabbitMQ' });
      }

      conn.createChannel(function (err, ch) {
        if (err) {
          console.error(err);
          return res.status(500).json({ error: 'Failed to create RabbitMQ channel' });
        }

        const queue = process.env?.DEFAULT_QUEUE_NAME || 'blockchain-data';
        const message = value;

        // ch.assertQueue(queue, { durable: false });
        ch.sendToQueue(queue, Buffer.from(message));

        console.log(`Sent message "${message}" to queue "${queue}"`);

        return res.status(200).json({ message: `Sent "${message}" to queue "${queue}"` });
      });
    });
  } catch (err) {
    console.error(err);
    return res.status(500).json({ error: 'Internal server error' });
  }
}
