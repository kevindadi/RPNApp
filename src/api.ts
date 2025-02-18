import axios, { AxiosInstance } from 'axios';

const HTTP_ENDPOINT = 'http://106.14.126.126:8080/';

const api: AxiosInstance = axios.create({
  baseURL: HTTP_ENDPOINT,
  headers: {
    'Content-Type': 'application/json',
  },
  timeout: 10000 // 10 seconds timeout
});

// Interfaces for request payloads
interface RustSourceRequest {
  source_code: string;
}

interface ListFilesRequest {
  dir_path: string;
}

interface PnAnalysisRequest {
  source_code: string;
  mode: string;
}

interface FileContentRequest {
  file_path: string;
}

// Interfaces for response payloads
interface MirResponse {
  success: boolean;
  content: string;
}

interface ListFilesResponse {
  filenames: string[];
}

interface PnAnalysisResponse {
  graph_content: string;
  output: string;
  error: string;
}

interface FileContentResponse {
  content: string;
}

// API Functions
export const getMir = async (sourceCode: string): Promise<MirResponse> => {
  try {
    const response = await api.post<MirResponse>('/get_mir', { source_code: sourceCode });
    return response.data;
  } catch (error) {
    console.error('Error in getMir:', error);
    throw error;
  }
};

export const listFiles = async (dirPath: string): Promise<ListFilesResponse> => {
  try {
    const response = await api.post<ListFilesResponse>('/list_files', { dir_path: dirPath });
    return response.data;
  } catch (error) {
    console.error('Error in listFiles:', error);
    throw error;
  }
};

export const runPnAnalysis = async (sourceCode: string, mode: string): Promise<PnAnalysisResponse> => {
  try {
    const response = await api.post<PnAnalysisResponse>('/run_pn_analysis', { source_code: sourceCode, mode: mode });
    return response.data;
  } catch (error) {
    console.error('Error in runPnAnalysis:', error);
    throw error;
  }
};

export const getFileContent = async (filePath: string): Promise<FileContentResponse> => {
  try {
    const response = await api.post<FileContentResponse>('/get_file_content', { file_path: filePath });
    return response.data;
  } catch (error) {
    console.error('Error in getFileContent:', error);
    throw error;
  }
};