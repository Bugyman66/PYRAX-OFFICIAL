import axios from 'axios';
import { StatusResponse } from './types';

// Fetch from Next.js API Proxy
export const fetchStatus = async (): Promise<StatusResponse> => {
    const { data } = await axios.get<StatusResponse>('/api/status');
    return data;
};
