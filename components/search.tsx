"use client";

import React, { useEffect, useState } from "react";

import { Input } from "@/components/ui/input";
import { useDebounce } from "@uidotdev/usehooks";
import { invoke } from "@tauri-apps/api";

import { open } from "@tauri-apps/api/shell";

import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

import SelectDirectory from "@/components/select-directory";
import SavedDirectories from "./saved-directories";

type SearchProps = {
  search: string;
  setSearch: (search: string) => void;
};

const SearchInput = ({ search, setSearch }: SearchProps) => {
  return (
    <Input
      type="text"
      value={search}
      onChange={(e) => setSearch(e.target.value)}
      placeholder="Type to find files..."
    />
  );
};

type SearchResult = {
  score: number;
  doc: {
    filename: string;
    content: string;
  };
};

type SearchResultProps = {
  results: SearchResult[];
};

const SearchResult = ({ results }: SearchResultProps) => {
  if (results.length === 0) {
    return <div></div>;
  }

  const openFileInFileExplorer = async (filename: string) => {
    invoke("openfile", { path: filename }).then(console.log).catch(console.error);
  };

  return (
    <Table>
      <TableCaption>{`Showing top ${results.length} results`}</TableCaption>
      <TableHeader>
        <TableRow>
          <TableHead className="w-[100px]">Filename</TableHead>
          <TableHead className="text-right">Match Score</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {results.map((res, index) => (
          <TableRow key={index}>
            {/* TODO: Why is filename a list? */}
            <TableCell onClick={() => openFileInFileExplorer(res.doc.filename[0])} className="font-medium text-left hover:cursor-pointer">
              {res.doc.filename}
            </TableCell>
            <TableCell className="text-right">{res.score.toFixed(2)}</TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
};

const Search = () => {
  const [search, setSearch] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);

  const debouncedSearch = useDebounce(search, 300);

  useEffect(() => {
    if (!debouncedSearch) {
      setResults([]);
      return;
    }

    invoke<{ score: number; doc: string }[]>("search", {
      query: debouncedSearch,
    })
      .then((searchResults) => {
        const results: SearchResult[] = [];
        searchResults.forEach((res) => {
          results.push({
            score: res.score,
            doc: JSON.parse(res.doc),
          });
        });
        setResults(results);
      })
      .catch(console.error);
  }, [debouncedSearch]);

  return (
    <>
      <div className="flex justify-end mt-3 space-x-6">
        <SavedDirectories />
        <SelectDirectory />
      </div>
      <h1 className="text-2xl font-bold">Find files</h1>
      <div className="flex items-center justify-between mt-3">
        <SearchInput search={search} setSearch={setSearch} />
      </div>
      {results && (
        <div>
          <SearchResult results={results} />
        </div>
      )}
    </>
  );
};

export default Search;
