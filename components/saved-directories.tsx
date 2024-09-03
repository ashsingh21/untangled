"use client";

import { useState } from "react";
import { CaretSortIcon } from "@radix-ui/react-icons";

import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { useAtomValue } from "jotai";
import { directoriesAtom } from "@/lib/atoms";
import DeleteIndex from "./ui/delete-alert";


export default function SavedDirectories() {
  const [open, setOpen] = useState(false);

  const directories = useAtomValue(directoriesAtom);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className="w-[200px] justify-between"
        >
          {"Indexed folders"}
          <CaretSortIcon className="ml-2 h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] p-0">
        <Command>
          <CommandInput placeholder="Search folder..." className="h-9" />
          <CommandList>
            <CommandEmpty>No indexed folder.</CommandEmpty>
            <CommandGroup>
              {directories.map((directory) => (
              <CommandItem key={directory} value={directory}>
                <div className="flex justify-between items-center w-full">
                  <span>{directory.split("/").pop()}</span>
                  <DeleteIndex directory={directory} />
                </div>
              </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
