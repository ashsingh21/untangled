import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger,
  } from "@/components/ui/alert-dialog"
  import { Button } from "@/components/ui/button"
import { directoriesAtom } from "@/lib/atoms";
import { useSetAtom } from "jotai";

type DeleteIndexProps = {
    directory: string;
}

export function DeleteIndex({ directory }: DeleteIndexProps) {
    const setDirectories = useSetAtom(directoriesAtom);

    const handleDelete = () => {
        console.log("Deleting ", directory);
        setDirectories((dirs) => dirs.filter((dir) => dir !== directory));
    }

    return (
        <AlertDialog>
            <AlertDialogTrigger asChild>
                <Button className="text-red-400 hover:text-red-600" variant="ghost">X</Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
                <AlertDialogHeader>
                <AlertDialogTitle>Are you sure?</AlertDialogTitle>
                <AlertDialogDescription>
                    This will remove the folder from the list of indexed folders.
                </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction onClick={handleDelete}>Continue</AlertDialogAction>
                </AlertDialogFooter>
            </AlertDialogContent>
        </AlertDialog>
    )
}

  export default DeleteIndex;