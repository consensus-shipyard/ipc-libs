'use client'

import Link from 'next/link';
import { useState, useEffect } from 'react';
import Head from 'next/head';

import SubnetStatus from './status';
import DashboardPage from './status';

export default function Home() {
  return (
    <div className="flex h-screen bg-gray-100">
      {/* Sidebar
      <aside className="w-64 bg-blue-500 p-4">
        <h1 className="text-white text-2xl font-semibold">Data Dashboard</h1>
        <nav className="mt-4">
          <ul>
            <li className="mb-2">
              <a href="#" className="text-white hover:text-gray-300">Overview</a>
            </li>
            <li className="mb-2">
              <a href="#" className="text-white hover:text-gray-300">Analytics</a>
            </li>
            <li className="mb-2">
              <a href="#" className="text-white hover:text-gray-300">Reports</a>
            </li>
          </ul>
        </nav>
      </aside> */}

      <DashboardPage />

    </div>
  );
}
