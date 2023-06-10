'use client'
import styles from './page.module.css'
import React, { useState } from 'react'

export default function MainPage() {
    const [message, setMessage] = useState('')
    const [range, setRange] = useState('')
    const [blocks, setBlocks] = useState([])
    const [statusMessage, setStatusMessage] = useState('')

    const handleValueChange = event => setMessage(event.target.value)

    const handleValueClick = async () => {
        if (!message) return
        setStatusMessage('Sending message...')
        try {
            const response = await fetch(`/api/rabbit_put?value=${encodeURIComponent(message)}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json'
                }
            })

            if (response.ok) setStatusMessage('Message sent successfully.')
            else setStatusMessage('Failed to send message.')
        } catch (error) {
            console.error('Error:', error)
        }
    }

    const handleRangeChange = event => setRange(event.target.value)

    const handleRangeClick = async () => {
        setStatusMessage('Fetching blocks...')
        try {
            let rangeV = encodeURIComponent(range)
            if (!rangeV) rangeV = '0'
            const blocksResponse = await fetch(`/api/redis_range?range=${rangeV}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json'
                }
            })
            const blocksData = await blocksResponse.json()
            if (!blocksData['blocks'].length) {
                setStatusMessage('Blocks not found.')
                return
            }
            setBlocks(blocksData['blocks'])
            setStatusMessage('Blocks found:')
        } catch (error) {
            console.error('Error:', error)
            setStatusMessage('Failed to fetch blocks.')
        }
    }

    return (
        <div className={styles.container}>
            <input className={styles.input} type="text" value={message} onChange={handleValueChange} placeholder="Message" />
            <button className={styles.button} onClick={handleValueClick}>
                Sign message
            </button>
            <input className={styles.input} type="number" value={range} onChange={handleRangeChange} placeholder="Range" />
            <button className={styles.button} onClick={handleRangeClick}>
                Get blocks
            </button>
            {statusMessage && <p className={styles.statusMessage}>{statusMessage}</p>}
            <ul className={styles.blocks}>
                {blocks.map((response, index) => (
                    <li className={styles.block} key={index}>
                        {response}
                    </li>
                ))}
            </ul>
        </div>
    )
}
